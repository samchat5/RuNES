use crate::apu::base_channel::AudioChannel;

use super::{envelope::Envelope, length_counter::LengthCounter, sweep::Sweep};

const EIGHTH: [u8; 8] = [0, 1, 0, 0, 0, 0, 0, 0];
const QUARTER: [u8; 8] = [0, 1, 1, 0, 0, 0, 0, 0];
const HALF: [u8; 8] = [0, 1, 1, 1, 1, 0, 0, 0];
const QUARTER_NEG: [u8; 8] = [1, 0, 0, 1, 1, 1, 1, 1];
const DUTY_CYCLES: [[u8; 8]; 4] = [EIGHTH, QUARTER, HALF, QUARTER_NEG];

pub struct Pulse {
    // duty: u8,
    // duty_pos: u8,
    // sweep_enabled: bool,
    // sweep_period: u8,
    // sweep_negate: bool,
    // sweep_shift: u8,
    // reload_sweep: bool,
    // sweep_divider: u8,
    // sweep_target_period: u32,
    // real_period: u16,
    // pub(crate) envelope: Envelope,
    // is_pulse_1: bool,
    enabled: bool,
    duty_cycle: u8,
    duty_counter: u8,
    freq_timer: u16,
    freq_counter: u16,
    channel: AudioChannel,
    length: LengthCounter,
    envelope: Envelope,
    sweep: Sweep,
}

impl Pulse {
    pub(crate) fn new(channel: AudioChannel) -> Self {
        Self {
            enabled: false,
            duty_cycle: 0,
            duty_counter: 0,
            freq_timer: 0,
            freq_counter: 0,
            channel,
            length: LengthCounter::new(),
            envelope: Envelope::new(),
            sweep: Sweep::default(),
        }
    }

    pub fn length_counter(&self) -> u8 {
        self.length.counter()
    }

    pub fn clock_quarter_frame(&mut self) {
        self.envelope.clock();
    }

    pub fn clock_half_frame(&mut self) {
        let sweep_forcing_silence = self.sweep_forcing_silence();
        let mut swp = &mut self.sweep;
        if swp.reload {
            swp.counter = swp.timer;
            swp.reload = false;
        } else if swp.counter > 0 {
            swp.counter -= 1;
        } else {
            swp.counter = swp.timer;
            if swp.enabled && !sweep_forcing_silence {
                let delta = self.freq_timer >> swp.shift;
                if swp.negate {
                    self.freq_timer -= delta + 1;
                    if self.channel == AudioChannel::Pulse1 {
                        self.freq_timer += 1;
                    }
                } else {
                    self.freq_timer += delta;
                }
            }
        }

        self.length.clock();
    }

    fn sweep_forcing_silence(&self) -> bool {
        let next = self.freq_timer + (self.freq_timer >> self.sweep.shift);
        self.freq_timer < 8 || (!self.sweep.negate && next > 0x7FF)
    }

    pub fn output(&self) -> f32 {
        if DUTY_CYCLES[self.duty_cycle as usize][self.duty_counter as usize] != 0
            && self.length.counter != 0
            && !self.sweep_forcing_silence()
        {
            if self.envelope.enabled {
                self.envelope.volume as f32
            } else {
                self.envelope.constant_volume as f32
            }
        } else {
            0.0
        }
    }

    pub fn write_ctrl(&mut self, val: u8) {
        self.duty_cycle = (val >> 6) & 0x3;
        self.length.write_ctrl(val);
        self.envelope.write_ctrl(val);
    }

    pub fn write_sweep(&mut self, val: u8) {
        self.sweep.timer = (val >> 4) & 0x7;
        self.sweep.negate = (val >> 3) & 0x1 == 0x1;
        self.sweep.shift = val & 0x7;
        self.sweep.enabled = ((val >> 7) & 1 == 1) && (self.sweep.shift != 0);
        self.sweep.reload = true;
    }

    pub fn write_timer_lo(&mut self, val: u8) {
        self.freq_timer = (self.freq_timer & 0xFF00) | val as u16;
    }

    pub fn write_timer_hi(&mut self, val: u8) {
        self.freq_timer = (self.freq_timer & 0x00FF) | ((val as u16 & 0x7) << 8);
        self.freq_counter = self.freq_timer;
        self.duty_counter = 0;
        self.envelope.reset = true;
        if self.enabled {
            self.length.load_value(val);
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length.counter = 0;
        }
    }

    pub fn clock(&mut self) {
        if self.freq_counter > 0 {
            self.freq_counter -= 1;
        } else {
            self.freq_counter = self.freq_timer;
            self.duty_counter = (self.duty_counter + 1) & 0x7;
        }
    }

    // fn is_muted(&self) -> bool {
    //     self.real_period < 8 || (!self.sweep_negate && self.sweep_target_period > 0x7FF)
    // }

    // fn initialize_sweep(&mut self, val: u8) {
    //     self.sweep_enabled = val & 0x80 == 0x80;
    //     self.sweep_period = ((val & 0x70) >> 4) + 1;
    //     self.sweep_negate = val & 0x8 == 0x8;
    //     self.sweep_shift = val & 0x7;
    //     self.update_target_period();
    //     self.reload_sweep = true;
    // }

    // fn update_target_period(&mut self) {
    //     let shift_res = self.real_period >> self.sweep_shift;
    //     if self.sweep_negate {
    //         self.sweep_target_period = u32::from(self.real_period - shift_res);
    //         if self.is_pulse_1 {
    //             self.sweep_target_period = self.sweep_target_period.wrapping_sub(1);
    //         }
    //     } else {
    //         self.sweep_target_period = u32::from(self.real_period + shift_res);
    //     }
    // }

    // fn set_period(&mut self, new: u16) {
    //     self.real_period = new;
    //     self.envelope.length_counter.base_channel.period = (self.real_period * 2) + 1;
    //     self.update_target_period();
    // }

    // pub fn get_output(&mut self) -> f32 {
    //     f32::from(if self.is_muted() {
    //         0
    //     } else {
    //         DUTY_CYCLES[self.duty as usize][self.duty_pos as usize] * self.envelope.get_volume()
    //     })
    // }

    // fn clock(&mut self) {
    //     self.duty_pos = (self.duty_pos.wrapping_sub(1)) & 0x7;
    // }

    // fn reset(&mut self, soft_reset: bool) {
    //     self.envelope.reset(soft_reset);
    //     self.duty_pos = 0;
    //     self.duty_pos = 0;
    //     self.real_period = 0;
    //     self.sweep_enabled = false;
    //     self.sweep_period = 0;
    //     self.sweep_negate = false;
    //     self.sweep_shift = 0;
    //     self.reload_sweep = false;
    //     self.sweep_divider = 0;
    //     self.sweep_target_period = 0;
    //     self.update_target_period();
    // }

    // pub(crate) fn run(&mut self, cyc: u32) {
    //     let mut to_run = (cyc - self.envelope.length_counter.base_channel.prev_cycle) as i32;
    //     while to_run > i32::from(self.envelope.length_counter.base_channel.timer) {
    //         let inc_dec = self.envelope.length_counter.base_channel.timer + 1;
    //         to_run -= i32::from(inc_dec);
    //         self.envelope.length_counter.base_channel.prev_cycle += u32::from(inc_dec);
    //         self.clock();
    //         self.envelope.length_counter.base_channel.timer =
    //             self.envelope.length_counter.base_channel.period;
    //     }
    //     if to_run > 0 {
    //         self.envelope.length_counter.base_channel.timer -= to_run.unsigned_abs() as u16;
    //     } else {
    //         self.envelope.length_counter.base_channel.timer += to_run.unsigned_abs() as u16;
    //     }
    //     self.envelope.length_counter.base_channel.prev_cycle = cyc;
    // }

    // // Pulse channels: we call self.run() before write
    // // Returns need_to_run that APU captures to set internal flag when calling this function
    // pub(crate) fn write(&mut self, addr: u16, val: u8) -> bool {
    //     let mut need_to_run = false;
    //     match addr & 0x03 {
    //         0 => {
    //             let to_set = self
    //                 .envelope
    //                 .length_counter
    //                 .initialize((val & 0x20) == 0x20);
    //             if to_set {
    //                 need_to_run = true;
    //             }
    //             self.envelope.initialize(val);
    //             self.duty = (val & 0xc0) >> 6;
    //         }
    //         1 => self.initialize_sweep(val),
    //         2 => self.set_period((self.real_period & 0x700) | u16::from(val)),
    //         3 => {
    //             let to_set = self.envelope.length_counter.load(val);
    //             if to_set {
    //                 need_to_run = true;
    //             }
    //             self.set_period((self.real_period & 0xFF) | ((u16::from(val) & 0x07) << 8));
    //             self.duty_pos = 0;
    //             self.envelope.reset_envelope();
    //         }
    //         _ => unreachable!(),
    //     }
    //     need_to_run
    // }

    // pub fn tick_sweep(&mut self) {
    //     self.sweep_divider = self.sweep_divider.wrapping_sub(1);
    //     if self.sweep_divider == 0 {
    //         if self.sweep_shift > 0
    //             && self.sweep_enabled
    //             && self.real_period >= 8
    //             && self.sweep_target_period <= 0x7ff
    //         {
    //             self.set_period(self.sweep_target_period as u16);
    //         }
    //         self.sweep_divider = self.sweep_period;
    //     }
    //     if self.reload_sweep {
    //         self.sweep_divider = self.sweep_period;
    //         self.reload_sweep = false;
    //     }
    // }

    // pub fn get_status(&self) -> bool {
    //     self.envelope.length_counter.counter > 0
    // }
}
