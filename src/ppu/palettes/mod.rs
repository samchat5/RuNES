use std::{fs::File, io::Read};

use itertools::Itertools;
use sdl2::pixels::Color;

pub struct Palette {
    pub system_palette: [Color; 0x40],
}

impl Default for Palette {
    fn default() -> Self {
        Self::from_file("src/ppu/palettes/ntscpalette.pal")
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
                    Color { r, g, b, a: 0xff }
                })
                .collect::<Vec<Color>>()
                .try_into()
                .unwrap(),
        }
    }
}
