pub struct Scroll {
    x: u8,
    y: u8,
    is_writing_x: bool,
}

impl Default for Scroll {
    fn default() -> Scroll {
        Scroll {
            x: 0,
            y: 0,
            is_writing_x: true,
        }
    }
}

impl Scroll {
    pub fn write(&mut self, val: u8) {
        if self.is_writing_x {
            self.x = val;
        } else {
            self.y = val;
        }
        self.is_writing_x = !self.is_writing_x;
    }

    pub fn get_x(&self) -> u8 {
        self.x
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }
}
