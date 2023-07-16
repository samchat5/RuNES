pub mod base_channel;
mod dmc;
mod envelope;
mod frame_counter;
mod length_counter;
mod mixer;
mod noise;
mod pulse;
mod sweep;
mod triangle;

use dmc::DMC;

use frame_counter::FrameCounter;

use noise::Noise;
use pulse::Pulse;

use crate::apu::mixer::mixer_value;
use triangle::Triangle;

use self::base_channel::AudioChannel;
use self::frame_counter::FrameCounterMode;

const SAMPLE_RATE: f32 = 44_100.0;
const CPU_CLOCK_SPEED: f32 = 21_477_272.0 / 12.0;
const CLOCKS_PER_SAMPLE: f32 = CPU_CLOCK_SPEED / SAMPLE_RATE;

pub struct APU {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: DMC,
    frame_counter: FrameCounter,
    output_buffer: Vec<f32>,
    irq_pending: bool,
    irq_disabled: bool,
    cycle: usize,
}

impl APU {
    pub fn new() -> Self {
        let output_buffer = Vec::new();
        Self {
            pulse1: Pulse::new(AudioChannel::Pulse1),
            pulse2: Pulse::new(AudioChannel::Pulse2),
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: DMC::default(),
            frame_counter: FrameCounter::default(),
            output_buffer,
            irq_pending: false,
            irq_disabled: false,
            cycle: 0,
        }
    }

    fn clock_frame_counter(&mut self) {
        let clock = self.frame_counter.clock();
        if self.frame_counter.mode == FrameCounterMode::FourStep
            && !self.irq_disabled
            && self.frame_counter.step >= 4
        {
            self.irq_pending = true;
        }
        match clock {
            1 | 3 => self.clock_quarter_frame(),
            2 | 5 => {
                self.clock_quarter_frame();
                self.clock_half_frame();
            }
            _ => {}
        }

        if self.frame_counter.update() && self.frame_counter.mode == FrameCounterMode::FiveStep {
            self.clock_quarter_frame();
            self.clock_half_frame();
        }
    }

    fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_quarter_frame();
        self.pulse2.clock_quarter_frame();
    }

    fn clock_half_frame(&mut self) {
        self.pulse1.clock_half_frame();
        self.pulse2.clock_half_frame();
    }

    pub fn write_ctrl(&mut self, channel: AudioChannel, val: u8) {
        match channel {
            AudioChannel::Pulse1 => self.pulse1.write_ctrl(val),
            AudioChannel::Pulse2 => self.pulse2.write_ctrl(val),
            _ => {}
        }
    }

    pub fn write_sweep(&mut self, channel: AudioChannel, val: u8) {
        match channel {
            AudioChannel::Pulse1 => self.pulse1.write_sweep(val),
            AudioChannel::Pulse2 => self.pulse2.write_sweep(val),
            _ => {}
        }
    }

    pub fn write_timer_lo(&mut self, channel: AudioChannel, val: u8) {
        match channel {
            AudioChannel::Pulse1 => self.pulse1.write_timer_lo(val),
            AudioChannel::Pulse2 => self.pulse2.write_timer_lo(val),
            _ => {}
        }
    }

    pub fn write_timer_hi(&mut self, channel: AudioChannel, val: u8) {
        match channel {
            AudioChannel::Pulse1 => self.pulse1.write_timer_hi(val),
            AudioChannel::Pulse2 => self.pulse2.write_timer_hi(val),
            _ => {}
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let mut status = 0x0;
        if self.pulse1.length_counter() > 0 {
            status |= 0x1;
        }
        if self.pulse2.length_counter() > 0 {
            status |= 0x2;
        }
        if self.irq_pending {
            status |= 0x40;
        }
        self.irq_pending = false;
        status
    }

    pub fn write_status(&mut self, val: u8) {
        self.pulse1.set_enabled(val & 0x1 != 0);
        self.pulse2.set_enabled(val & 0x2 != 0);
    }

    pub fn write_frame_counter(&mut self, val: u8) {
        self.frame_counter.write(val, self.cycle);
        self.irq_disabled = val & 0x40 != 0;
        if self.irq_disabled {
            self.irq_pending = false;
        }
    }

    pub fn clock(&mut self) -> bool {
        if self.cycle & 0x01 == 0 {
            self.pulse1.clock();
            self.pulse2.clock();
        }
        self.clock_frame_counter();

        let s1 = (self.cycle as f32 / CLOCKS_PER_SAMPLE) as usize;
        self.cycle += 1;
        let s2 = (self.cycle as f32 / CLOCKS_PER_SAMPLE) as usize;
        if s1 != s2 {
            self.output_buffer.push(self.output());
        }

        self.irq_pending
    }

    fn output(&self) -> f32 {
        let pulse1 = self.pulse1.output();
        let pulse2 = self.pulse2.output();
        mixer_value(pulse1, pulse2, 0.0, 0.0, 0.0)
    }

    pub fn get_buffer(&self) -> &[f32] {
        &self.output_buffer
    }

    pub fn clear_buffer(&mut self) {
        self.output_buffer.clear();
    }
}
