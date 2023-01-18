use self::{address::Address, control::Control};

pub mod address;
pub mod control;

pub struct PPU {
    addr: Address,
    ctrl: Control,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            addr: Address::new(),
            ctrl: Control::new(),
        }
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        self.addr.write(val);
    }

    pub fn write_ppuctrl(&mut self, val: u8) {
        self.ctrl.write(val);
    }
}
