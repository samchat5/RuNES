use crate::apu::base_channel::AudioChannel;

use super::{envelope::Envelope, length_counter::{LengthCounter, NeedToRunFlag}, sweep::Sweep};

const EIGHTH: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const QUARTER: [u8; 8] = [0, 0, 0, 0, 0, 0, 1, 1];
const HALF: [u8; 8] = [0, 0, 0, 0, 1, 1, 1, 1];
const QUARTER_NEG: [u8; 8] = [1, 1, 1, 1, 1, 1, 0, 0];
const DUTY_CYCLES: [[u8; 8]; 4] = [EIGHTH, QUARTER, HALF, QUARTER_NEG];

pub struct Pulse {
    channel: AudioChannel,
    length: LengthCounter,
    envelope: Envelope,
    sweep: Sweep,
    real_period: u16,
    period: u16,
    previous_cycle: u64,
    timer: u16,
    duty_pos: u8,
    duty: u8
}

impl Pulse {
    pub fn new(channel: AudioChannel) -> Self {
        Self {
            channel,
            length: LengthCounter::new(),
            envelope: Envelope::new(),
            sweep: Sweep::default(),
            real_period: 0,
            period: 0,
            previous_cycle: 0,
            timer: 0,
            duty_pos: 0,
            duty: 0,
        }
    }

    pub fn output(&self) -> u8 {
        if self.is_muted() {
            return 0;
        }
        return (DUTY_CYCLES[self.duty as usize][self.duty_pos as usize] as u32 * self.get_volume()) as u8;
    }

    pub fn clock_envelope(&mut self) {
        self.envelope.clock(self.length.halt);
    }

    pub fn clock_length_counter(&mut self) {
        self.length.clock();
    }

    pub fn clock_sweep(&mut self) {
        self.sweep.divider = self.sweep.divider.wrapping_sub(1);
        if self.sweep.divider == 0 {
            if self.sweep.shift > 0
                && self.sweep.enabled
                && self.real_period >= 8
                && self.sweep.target_period <= 0x7ff
            {
                self.set_period((self.sweep.target_period & 0xFFFF) as u16);
            }
            self.sweep.divider = self.sweep.period;
        }

        if self.sweep.reload {
            self.sweep.divider = self.sweep.period;
            self.sweep.reload = false;
        }
    }

    pub fn reload_counter(&mut self) {
        self.length.reload();
    }

    pub fn clock(&mut self, target_cycle: u64) {
        let mut cycles_to_run = target_cycle - self.previous_cycle;
        while cycles_to_run > self.timer as u64 {
            cycles_to_run -= self.timer as u64 + 1;
            self.previous_cycle += self.timer as u64 + 1;
            self.duty_pos = (self.duty_pos.wrapping_sub(1)) & 0x07;
            self.timer = self.period;
        }
        self.timer -= cycles_to_run as u16;
        self.previous_cycle = target_cycle;
    }

    pub fn get_status(&self) -> bool {
        return self.length.counter > 0;
    } 
    
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.length.counter = 0;
        }
        self.length.enabled = enabled;
    }

    pub fn write_ctrl(&mut self, val: u8) -> NeedToRunFlag {
        let flag = self.length.init((val & 0x20) == 0x20);
        self.envelope.init(val);
        self.duty = (val & 0xc0) >> 6;
        return flag;

    }
    
    pub fn write_sweep(&mut self, val: u8) {
        self.sweep.enabled = (val & 0x80) == 0x80;
        self.sweep.negate = (val & 0x08) == 0x08;
        self.sweep.period = ((val & 0x70) >> 4) + 1;
        self.sweep.shift = val & 0x07;

        self.update_target_period();

        self.sweep.reload = true;
    }

    pub fn write_timer_lo(&mut self, val: u8) {
        self.set_period((self.real_period & 0x0700) | val as u16)
    }

    pub fn write_timer_hi(&mut self, val: u8) -> NeedToRunFlag {
        let flag = self.length.load(val >> 3);
        self.set_period((self.real_period & 0xff) | ((val as u16 & 0x07) << 8));
        self.duty_pos = 0;
        self.envelope.reset();
        return flag;
    }

    fn get_volume(&self) -> u32 {
        if self.length.counter > 0 {
            if self.envelope.constant_volume {
                return self.envelope.volume as u32;
            } 
            return self.envelope.counter as u32;
        } 
        return 0;
    }

    fn is_muted(&self) -> bool {
        return self.real_period < 8 || (!self.sweep.negate && self.sweep.target_period > 0x7ff);
    }

    fn set_period(&mut self, new_period: u16) {
        self.real_period = new_period;
        self.period = self.real_period * 2 + 1;
        self.update_target_period();
    }

    fn update_target_period(&mut self) {
        let shift_result = self.real_period >> self.sweep.shift;
        if self.sweep.negate {
            self.sweep.target_period = (self.real_period - shift_result) as u32;
            if self.channel == AudioChannel::Pulse1 {
                self.sweep.target_period = self.sweep.target_period.wrapping_sub(1);
            }
        } else {
            self.sweep.target_period = (self.real_period + shift_result) as u32;
        }
    }
}
