const LENGTH_LOOKUP: [u8; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub struct NeedToRunFlag(pub Option<bool>);

#[derive(Default)]
pub struct LengthCounter {
    pub enabled: bool,
    pub counter: u8,
    halt: bool,
    reload_val: u8,
    prev_value: u8,
    new_halt_val: bool,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter::default()
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

    pub fn load_value(&mut self, val: u8) -> NeedToRunFlag {
        self.reload_val = LENGTH_LOOKUP[(val >> 3) as usize];
        self.prev_value = self.counter;
        return NeedToRunFlag(Some(true));
    }

    pub fn write_ctrl(&mut self, val: u8) -> NeedToRunFlag {
        self.new_halt_val = (val >> 5) & 1 == 1;
        return NeedToRunFlag(Some(true));
    }

    pub fn clock(&mut self) {
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }
    }
}
