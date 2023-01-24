use bitflags::bitflags;

bitflags! {
    pub struct Control : u8 {
        const NAMETABLE_1 =             0b0000_0001;
        const NAMETABLE_2 =             0b0000_0010;
        const INCREMENT =               0b0000_0100;
        const SPRITE_PATTERN_ADDR =     0b0000_1000;
        const BACKGROUND_PATTERN_ADDR = 0b0001_0000;
        const SPRITE_SIZE =             0b0010_0000;
        const MASTER_SLAVE =            0b0100_0000;
        const NMI =                     0b1000_0000;
    }
}

impl Default for Control {
    fn default() -> Self {
        Self::empty()
    }
}

impl Control {
    pub fn new() -> Control {
        Control::empty()
    }

    pub fn write(&mut self, val: u8) {
        self.bits = val;
    }
}
