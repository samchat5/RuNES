pub struct Address {
    hi: u8,
    lo: u8,
    writing_hi: bool,
}

impl Address {
    pub fn new() -> Address {
        Address {
            hi: 0,
            lo: 0,
            writing_hi: false,
        }
    }

    pub fn write(&mut self, val: u8) {
        if self.writing_hi {
            self.hi = val;
        } else {
            self.lo = val;
        }
        self.writing_hi = !self.writing_hi;
    }

    pub fn read(&self) -> u16 {
        (self.hi as u16) << 8 | self.lo as u16
    }
}
