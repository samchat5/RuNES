const LENGTH_LOOKUP: [u8; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

#[derive(Default)]
pub struct LengthCounter {
    pub enabled: bool,
    pub(crate) counter: u8,
}

impl LengthCounter {
    pub(crate) fn new() -> LengthCounter {
        LengthCounter::default()
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn load_value(&mut self, val: u8) {
        self.counter = LENGTH_LOOKUP[(val >> 3) as usize];
    }

    pub fn write_ctrl(&mut self, val: u8) {
        self.enabled = (val >> 5) & 1 == 0;
    }

    pub fn clock(&mut self) -> usize {
        if self.enabled && self.counter > 0 {
            self.counter -= 1;
            1
        } else {
            0
        }
    }
}
