use std::{fs::File, io::Read};

use image::Rgb;
use itertools::Itertools;

pub struct Palette {
    pub system_palette: [Rgb<u8>; 0x40],
}

impl Default for Palette {
    fn default() -> Self {
        Self::from_file("src/core/ppu/palettes/ntscpalette.pal")
    }
}

impl Palette {
    pub fn from_file(path: &str) -> Palette {
        let file = File::open(path).unwrap();
        let reader = file.bytes();
        Palette {
            system_palette: reader
                .chunks(3)
                .into_iter()
                .map(|mut chunk| {
                    let r = chunk.next().unwrap().unwrap();
                    let g = chunk.next().unwrap().unwrap();
                    let b = chunk.next().unwrap().unwrap();
                    Rgb([r, g, b])
                })
                .collect::<Vec<Rgb<u8>>>()
                .try_into()
                .unwrap(),
        }
    }
}
