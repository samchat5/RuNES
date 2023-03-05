use bitflags::bitflags;

bitflags! {
    pub struct Buttons: u8 {
        const A =       0b0000_0001;
        const B =       0b0000_0010;
        const SELECT =  0b0000_0100;
        const START =   0b0000_1000;
        const UP =      0b0001_0000;
        const DOWN =    0b0010_0000;
        const LEFT =    0b0100_0000;
        const RIGHT =   0b1000_0000;
    }
}

pub struct Joypad {
    is_strobe_on: bool,
    button_idx: u8,
    pub buttons: Buttons,
}

impl Default for Joypad {
    fn default() -> Self {
        Joypad {
            is_strobe_on: false,
            button_idx: 0,
            buttons: Buttons::empty(),
        }
    }
}

impl Joypad {
    pub fn write(&mut self, data: u8) {
        self.is_strobe_on = data & 1 == 1;
        if self.is_strobe_on {
            self.button_idx = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_idx > 7 {
            return 1;
        }
        let ret = (self.buttons.bits & (1 << self.button_idx)) >> self.button_idx;
        if !self.is_strobe_on && self.button_idx <= 7 {
            self.button_idx += 1;
        }
        ret
    }
}
