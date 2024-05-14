pub mod base_channel;
pub mod dmc;
pub mod envelope;
pub mod frame_counter;
pub mod length_counter;
pub mod noise;
pub mod pulse;
pub mod sweep;
pub mod triangle;

use dmc::DMC;
use frame_counter::FrameCounter;
use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

use crate::frontend::blip_buf::BlipBuf;

use self::base_channel::AudioChannel;
use self::frame_counter::{FrameType, IRQSignal};
use self::length_counter::NeedToRunFlag;

pub struct APU {
    pulse1: Pulse,
    pulse2: Pulse,
    _triangle: Triangle,
    _noise: Noise,
    pub dmc: DMC,
    frame_counter: FrameCounter,
    pub output_buffer: BlipBuf<65536>,
    irq_pending: bool,
    irq_disabled: bool,
    cycle: usize,
    need_to_run: bool,
    prev_cycle: usize,
    need_dmc_transfer: bool,
}

impl Default for APU {
    fn default() -> Self {
        Self::new()
    }
}

impl APU {
    pub const CLOCK_RATE: f64 = 1789772.7272;
    const DEFAULT_SAMPLE_RATE: f64 = 48000.;

    #[must_use]
    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(AudioChannel::Pulse1),
            pulse2: Pulse::new(AudioChannel::Pulse2),
            _triangle: Triangle::default(),
            _noise: Noise::default(),
            dmc: DMC::default(),
            frame_counter: FrameCounter::default(),
            output_buffer: BlipBuf::new(Self::CLOCK_RATE, Self::DEFAULT_SAMPLE_RATE),
            irq_pending: false,
            irq_disabled: false,
            cycle: 0,
            need_to_run: false,
            prev_cycle: 0,
            need_dmc_transfer: false,
        }
    }

    #[must_use]
    pub const fn read_status_trace(&self) -> u8 {
        let mut status = 0;
        if self.pulse1.length.counter > 0 {
            status |= 0x1;
        }
        if self.pulse2.length.counter > 0 {
            status |= 0x2;
        }
        if self.irq_pending {
            status |= 0x40;
        }
        status
    }

    pub fn clock(&mut self) -> (bool, bool) {
        self.cycle += 1;
        // self.need_to_run();
        self.run();
        self.output();
        (self.irq_pending, self.need_dmc_transfer)
    }

    pub fn write_ctrl(&mut self, channel: &AudioChannel, val: u8) {
        let mut flag = NeedToRunFlag(None);
        match channel {
            AudioChannel::Pulse1 => flag = self.pulse1.write_ctrl(val),
            AudioChannel::Pulse2 => flag = self.pulse2.write_ctrl(val),
            _ => {}
        }

        if let Some(f) = flag.0 {
            self.need_to_run = f;
        }
    }

    pub fn write_sweep(&mut self, channel: &AudioChannel, val: u8) {
        match channel {
            AudioChannel::Pulse1 => self.pulse1.write_sweep(val),
            AudioChannel::Pulse2 => self.pulse2.write_sweep(val),
            _ => {}
        }
    }

    pub fn write_timer_lo(&mut self, channel: &AudioChannel, val: u8) {
        match channel {
            AudioChannel::Pulse1 => self.pulse1.write_timer_lo(val),
            AudioChannel::Pulse2 => self.pulse2.write_timer_lo(val),
            _ => {}
        }
    }

    pub fn write_timer_hi(&mut self, channel: &AudioChannel, val: u8) {
        let mut flag = NeedToRunFlag(None);
        match channel {
            AudioChannel::Pulse1 => flag = self.pulse1.write_timer_hi(val),
            AudioChannel::Pulse2 => flag = self.pulse2.write_timer_hi(val),
            _ => {}
        }

        if let Some(f) = flag.0 {
            self.need_to_run = f;
        }
    }

    fn run(&mut self) {
        let mut cycles_to_run = (self.cycle - self.prev_cycle) as i32;

        while cycles_to_run > 0 {
            let callback = |typ: FrameType| {
                self.pulse1.clock_quarter_frame();
                self.pulse2.clock_quarter_frame();
                if typ == FrameType::HalfFrame {
                    self.pulse1.clock_length_counter();
                    self.pulse2.clock_length_counter();
                    self.pulse1.clock_sweep();
                    self.pulse2.clock_sweep();
                }
            };

            let (signal, inc) =
                self.frame_counter
                    .clock(self.irq_disabled, &mut cycles_to_run, callback);
            self.prev_cycle += inc as usize;
            match signal {
                IRQSignal::Clear => self.irq_pending = false,
                IRQSignal::Set => self.irq_pending = true,
                IRQSignal::None => {}
            }

            self.pulse1.reload_counter();
            self.pulse2.reload_counter();

            self.pulse1.clock(self.prev_cycle as u64);
            self.pulse2.clock(self.prev_cycle as u64);
            self.need_dmc_transfer = self.dmc.clock(self.prev_cycle as u64);
        }
    }

    pub fn read_status(&mut self) -> (u8, IRQSignal) {
        let mut status = 0x0;
        if self.pulse1.length.counter > 0 {
            status |= 0x1;
        }
        if self.pulse2.length.counter > 0 {
            status |= 0x2;
        }
        if self.irq_pending {
            status |= 0x40;
        }
        if self.dmc.bytes_remaining > 0 {
            status |= 0x10;
        }
        self.irq_pending = false;
        (status, IRQSignal::Clear)
    }

    pub fn write_dmc_ctrl(&mut self, data: u8) {
        self.run();
        self.dmc.write_ctrl(data);
    }

    pub fn write_dmc_load(&mut self, data: u8) {
        self.run();
        self.dmc.write_load(data);
    }

    pub fn write_dmc_addr(&mut self, data: u8) {
        self.run();
        self.dmc.write_addr(data);
    }

    pub fn write_dmc_lc(&mut self, data: u8) {
        self.run();
        self.dmc.write_lc(data);
    }

    pub fn write_status(&mut self, val: u8, cpu_cycle: u64) {
        self.pulse1.set_enabled(val & 0x1 != 0);
        self.pulse2.set_enabled(val & 0x2 != 0);
        self.dmc.set_enabled(val & 0x10 != 0, cpu_cycle)
    }

    pub fn write_frame_counter(&mut self, val: u8) -> IRQSignal {
        self.frame_counter.write(val, self.cycle);
        self.irq_disabled = val & 0x40 != 0;
        if self.irq_disabled {
            self.irq_pending = false;
            return IRQSignal::Clear;
        }
        IRQSignal::None
    }

    fn output(&mut self) {
        let pulse1 = self.pulse1.output() as f64;
        let pulse2 = self.pulse2.output() as f64;
        let pulse_out = pulse1 + pulse2;
        let square_volume = (477600. / (8128.0 / pulse_out + 100.0)) as i32;
        self.output_buffer.add_sample(square_volume);
    }
}
