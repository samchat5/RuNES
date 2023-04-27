use sdl2::pixels::Color;

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub image: [u8; 256 * 240 * 3],
    pub is_zero: [[bool; 256]; 240],
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            image: [0; 256 * 240 * 3],
            is_zero: [[false; 256]; 240],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = (y * 256 + x) * 3;
        self.image[index] = color.r;
        self.image[index + 1] = color.g;
        self.image[index + 2] = color.b;
    }
}
