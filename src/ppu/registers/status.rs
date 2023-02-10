use bitflags::bitflags;

bitflags! {
    pub struct Status  : u8 {
        const VBLANK = 1 << 7;
        const SPRITE_ZERO_HIT = 1 << 6;
        const SPRITE_OVERFLOW = 1 << 5;
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
