use super::length_counter::{LengthCounter, NeedToRunFlag};

const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

#[derive(Default)]
pub struct LinearCounter {
    counter: u8,
    pub counter_reload: u8,
    pub reload: bool,
    pub control: bool,
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            counter_reload: 0,
            reload: false,
            control: false,
        }
    }

    pub fn clock(&mut self) {
        if self.reload {
            self.counter = self.counter_reload;
        } else if self.counter != 0 {
            self.counter -= 1;
        }
        if !self.control {
            self.reload = false;
        }
    }
}

#[derive(Default)]
pub struct Triangle {
    pub length: LengthCounter,
    linear: LinearCounter,
    timer: u16,
    previous_cycle: u64,
    period: u16,
    seq_pos: u8,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            length: LengthCounter::new(),
            linear: LinearCounter::new(),
            timer: 0,
            previous_cycle: 0,
            period: 0,
            seq_pos: 0,
        }
    }

    pub fn output(&self) -> f32 {
        if self.period < 2 {
            return 7.5;
        }
        f32::from(SEQUENCE[self.seq_pos as usize])
    }

    pub fn clock_quarter_frame(&mut self) {
        self.linear.clock();
    }

    pub fn clock_half_frame(&mut self) {
        self.length.clock();
    }

    pub fn reload_counter(&mut self) {
        self.length.reload();
    }

    pub fn write_ctrl(&mut self, data: u8) {
        self.linear.control = (data >> 7) == 1;
        self.linear.counter_reload = data & 0x8F;
        self.length.write_ctrl(data >> 7);
    }

    pub fn write_timer_lo(&mut self, data: u8) {
        self.period = self.period & 0xff00 | (data as u16);
    }

    pub fn write_timer_hi(&mut self, data: u8) -> NeedToRunFlag {
        self.period = self.period & 0x00ff | (((data & 0x07) as u16) << 8);
        self.linear.reload = true;
        if self.length.enabled {
            return self.length.load_value(data);
        }
        NeedToRunFlag(None)
    }

    pub fn clock(&mut self, target_cycle: u64) {
        let mut cycles_to_run = target_cycle - self.previous_cycle;
        while cycles_to_run > u64::from(self.timer) {
            cycles_to_run -= u64::from(self.timer) + 1;
            self.previous_cycle += u64::from(self.timer) + 1;
            self.timer = self.period;
            if self.length.counter > 0 && self.linear.counter > 0 {
                self.seq_pos = (self.seq_pos + 1) & 0x1F
            }
        }
        self.timer -= cycles_to_run as u16;
        self.previous_cycle = target_cycle;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.length.enabled = enabled;
    }
}
