#[derive(Default)]
pub struct Envelope {
    pub enabled: bool,
    loops: bool,
    pub reset: bool,
    pub volume: u8,
    pub constant_volume: u8,
    counter: u8,
}

impl Envelope {
    pub(crate) fn new() -> Envelope {
        Envelope::default()
    }

    pub fn write_ctrl(&mut self, val: u8) {
        self.loops = (val >> 5) & 1 == 1;
        self.enabled = (val >> 4) & 1 == 0;
        self.constant_volume = val & 0xf;
    }

    pub fn clock(&mut self) {
        if self.reset {
            self.reset = false;
            self.volume = 0xf;
            self.counter = self.constant_volume;
        } else if self.counter > 0 {
            self.counter -= 1;
        } else {
            self.counter = self.constant_volume;
            if self.volume > 0 {
                self.volume -= 1;
            } else if self.loops {
                self.volume = 0xf;
            }
        }
    }
}
