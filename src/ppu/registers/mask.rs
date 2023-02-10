use bitflags::bitflags;

bitflags! {
    pub struct Mask : u8 {
        const GREYSCALE = 1 << 0;
        const SHOW_LEFT_BACKGROUND = 1 << 1;
        const SHOW_LEFT_SPRITES = 1 << 2;
        const SHOW_BACKGROUND = 1 << 3;
        const SHOW_SPRITES = 1 << 4;
        const EMPHASIZE_RED = 1 << 5;
        const EMPHASIZE_GREEN = 1 << 6;
        const EMPHASIZE_BLUE = 1 << 7;
    }
}

impl Default for Mask {
    fn default() -> Mask {
        Mask::empty()
    }
}

impl Mask {
    pub fn new() -> Mask {
        Mask::empty()
    }

    pub fn write(&mut self, val: u8) {
        self.bits = val;
    }
}
