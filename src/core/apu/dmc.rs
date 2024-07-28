use super::frame_counter::IRQSignal;

const PERIOD_LOOKUP: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct DMC {
    /// Writeable values
    irq_enable: bool,
    _loop: bool,
    period: u16,
    pub output_level: u8,
    sample_addr: u16,
    sample_length: u16,

    // Private buffers/registers
    shift_register: u8,
    bits_remaining: u8,
    pub bytes_remaining: u16,
    pub current_addr: u16,
    need_init: u8,

    // Cycle/timer values
    previous_cycle: u64,
    timer: u16,

    // Output
    output_buffer: Option<u8>,

    /// Misc flags
    silence_flag: bool,
    need_to_run: bool,
}

impl Default for DMC {
    fn default() -> DMC {
        DMC {
            irq_enable: false,
            _loop: false,
            period: PERIOD_LOOKUP[0],
            sample_addr: 0,
            sample_length: 0,
            shift_register: 0,
            bits_remaining: 8,
            bytes_remaining: 0,
            current_addr: 0,
            need_init: 0,
            previous_cycle: 0,
            timer: PERIOD_LOOKUP[0],
            output_level: 0,
            output_buffer: None,
            silence_flag: true,
            need_to_run: false,
        }
    }
}

impl DMC {
    pub fn new() -> DMC {
        DMC::default()
    }

    // RAM Writes --------------------------------------------------------------
    pub fn write_ctrl(&mut self, data: u8) {
        self.irq_enable = data >> 7 != 0;
        self._loop = data >> 6 != 0;
        self.period = PERIOD_LOOKUP[(data & 0x0f) as usize];
    }

    pub fn write_load(&mut self, data: u8) {
        self.output_level = data & 0x7f;
    }

    pub fn write_addr(&mut self, data: u8) {
        self.sample_addr = (data as u16) * 64 + 0xc000;
    }
    pub fn write_lc(&mut self, data: u8) {
        self.sample_length = (data as u16) + 1;
    }
    // ------------------------------------------------------------------------

    pub fn clock(&mut self, target_cycle: u64) -> bool {
        let mut should_start_dmc_transfer = false;
        let mut cycles_to_run = target_cycle - self.previous_cycle;

        while cycles_to_run > u64::from(self.timer) {
            cycles_to_run -= u64::from(self.timer) + 1;
            self.previous_cycle += u64::from(self.timer) + 1;
            self.timer = self.period;

            if !self.silence_flag {
                if self.shift_register & 1 != 0 {
                    if self.output_level <= 125 {
                        self.output_level += 2;
                    }
                } else if self.output_level >= 2 {
                    self.output_level -= 1
                }
                self.shift_register >>= 1;
            }

            self.bits_remaining -= 1;
            if self.bits_remaining == 0 {
                self.bits_remaining = 8;
                self.silence_flag = self.output_buffer.is_none();
                if let Some(x) = self.output_buffer {
                    self.shift_register = x;
                    self.output_buffer = None;
                    should_start_dmc_transfer = self.should_start_dmc_transfer();
                }
            }
        }

        self.timer -= cycles_to_run as u16;
        self.previous_cycle = target_cycle;

        should_start_dmc_transfer
    }

    pub fn should_start_dmc_transfer(&mut self) -> bool {
        self.output_buffer.is_none() && self.bytes_remaining > 0
    }

    pub fn set_dmc_read_buffer(&mut self, val: u8) -> IRQSignal {
        if self.bytes_remaining > 0 {
            self.output_buffer = Some(val);

            if self.current_addr != 0xffff {
                self.current_addr += 1;
            } else {
                self.current_addr = 0x8000;
            }

            self.bytes_remaining -= 1;

            if self.bytes_remaining == 0 {
                self.need_to_run = false;
                if self._loop {
                    self.init_sample();
                } else if self.irq_enable {
                    return IRQSignal::Set;
                }
            }
        }
        IRQSignal::None
    }

    fn init_sample(&mut self) {
        self.current_addr = self.sample_addr;
        self.bytes_remaining = self.sample_length;
        self.need_to_run = self.bytes_remaining > 0;
    }

    pub fn set_enabled(&mut self, enabled: bool, cpu_cycle: u64) {
        if !enabled {
            self.bytes_remaining = 0;
            self.need_to_run = false;
        } else if self.bytes_remaining == 0 {
            self.init_sample();
            if cpu_cycle & 0x01 == 0 {
                self.need_init = 2;
            } else {
                self.need_init = 3;
            }
        }
    }

    pub fn need_to_run(&mut self) -> (bool, bool) {
        let mut should_start_dmc_transfer = false;

        if self.need_init > 0 {
            self.need_init -= 1;
            if self.need_init == 0 {
                should_start_dmc_transfer = self.should_start_dmc_transfer()
            }
        }

        (should_start_dmc_transfer, self.need_to_run)
    }

    pub fn irq_pending(&self, cycles_to_run: u64) -> bool {
        if self.irq_enable && self.bytes_remaining > 0 {
            let cycles_to_empty =
                (self.bits_remaining as u16 + ((self.bytes_remaining - 1) * 8)) * self.period;
            if cycles_to_run >= cycles_to_empty as u64 {
                return true;
            }
        }
        false
    }

    pub fn output(&self) -> f32 {
        self.output_level as f32
    }
}
