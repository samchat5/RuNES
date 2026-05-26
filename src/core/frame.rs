use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use image::{ImageResult, Rgb, save_buffer};
use image::ColorType;

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

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb<u8>) {
        let index = (y * 256 + x) * 3;
        self.image[index] = color.0[0];
        self.image[index + 1] = color.0[1];
        self.image[index + 2] = color.0[2];
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.image.hash(&mut hasher);
        hasher.finish()
    }

    pub fn save_buffer(&self, path: impl AsRef<Path>) -> ImageResult<()> {
        save_buffer(path, &self.image, 256, 240, ColorType::Rgb8)
    }
}
