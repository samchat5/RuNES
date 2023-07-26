const LENGTH_LOOKUP: [u8; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub struct NeedToRunFlag(pub Option<bool>); 

#[derive(Default)]
pub struct LengthCounter {
    pub enabled: bool,
    pub halt: bool,
    pub counter: u8,
    reload_val: u8,
    prev_value: u8,
    new_halt_val: bool,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter::default()
    }

    pub fn init(&mut self, halt: bool) -> NeedToRunFlag {
        self.new_halt_val = halt;
        return NeedToRunFlag(Some(true))
    }

    pub fn load(&mut self, val: u8) -> NeedToRunFlag {
        if self.enabled {
            self.reload_val = LENGTH_LOOKUP[val as usize];
            self.prev_value = self.counter;
            return NeedToRunFlag(Some(true))
        }
        return NeedToRunFlag(None)
    }

    pub fn get_status(&self) -> bool {
        self.counter > 0
    }

    pub fn reload(&mut self) {
        if self.reload_val != 0 {
            if self.counter == self.prev_value {
                self.counter = self.reload_val;
            }
            self.reload_val = 0;
        }
        self.halt = self.new_halt_val;
    }

    pub fn clock(&mut self) {
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !self.enabled {
            self.counter = 0;
        }
        self.enabled = enabled;
    }
}
