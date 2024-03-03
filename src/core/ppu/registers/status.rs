use bitflags::bitflags;

bitflags! {
    pub struct Status  : u8 {
        const VBLANK = 1 << 7;
        const SPRITE_ZERO_HIT = 1 << 6;
        const SPRITE_OVERFLOW = 1 << 5;
        const UNUSED4 = 1 << 4;
        const UNUSED3 = 1 << 3;
        const UNUSED2 = 1 << 2;
        const UNUSED1 = 1 << 1;
        const UNUSED0 = 1 << 0;
    }
}

impl Default for Status {
    fn default() -> Status {
        Status::empty()
    }
}

impl Status {
    pub fn new() -> Status {
        Status::empty()
    }
}
