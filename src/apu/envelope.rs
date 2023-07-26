#[derive(Default)]
pub struct Envelope {
    pub enabled: bool,
    pub reset: bool,
    pub constant_volume: bool,
    pub volume: u8,
    divider: i8,
    pub counter: u8,
    start: bool,
}

impl Envelope {
    pub(crate) fn new() -> Envelope {
        Envelope::default()
    }

    pub fn clock(&mut self, length_counter_halt: bool) {
        if !self.start {
            self.divider -= 1;
            if self.divider < 0 {
                self.divider = self.volume as i8;
                if self.counter > 0 {
                    self.counter -= 1;
                } else if length_counter_halt {
                    self.counter = 15;
                }
            }
        } else {
            self.start = false;
            self.counter = 15;
            self.divider = self.volume as i8
        }
    }

    pub fn init(&mut self, val: u8) {
        self.constant_volume = (val & 0x10) == 0x10;
        self.volume = val & 0x0F;
    }

    pub fn reset(&mut self) {
        self.start = true;
    }
}
