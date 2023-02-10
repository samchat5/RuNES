pub struct Address {
    hi: u8,
    lo: u8,
    writing_hi: bool,
}

impl Default for Address {
    fn default() -> Self {
        Self::new()
    }
}

impl Address {
    pub(in crate::ppu) fn new() -> Address {
        Address {
            hi: 0,
            lo: 0,
            writing_hi: true,
        }
    }

    fn set(&mut self, val: u16) {
        self.hi = (val >> 8) as u8;
        self.lo = (val & 0xff) as u8;
    }

    pub(in crate::ppu) fn write(&mut self, val: u8) {
        if self.writing_hi {
            self.hi = val;
        } else {
            self.lo = val;
        }

        if self.read() > 0x3fff {
            self.set(self.read() & 0x3fff);
        }

        self.writing_hi = !self.writing_hi;
    }

    pub(in crate::ppu) fn read(&self) -> u16 {
        (self.hi as u16) << 8 | self.lo as u16
    }

    pub(in crate::ppu) fn increment(&mut self, inc: u16) {
        let addr = self.read();
        self.set(addr + inc);
        if self.read() > 0x3fff {
            self.set(self.read() & 0x3fff);
        }
    }
}
