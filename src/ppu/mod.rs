use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use sdl2::pixels::Color;

use crate::{
    frame::Frame,
    mappers::{Mapper, Mirroring},
};

use self::registers::{address::Address, control::Control, mask::Mask, status::Status};

pub mod palettes;
mod registers;

pub struct PPU {
    addr: Address,
    ctrl: Control,
    status: Status,
    mask: Mask,

    name_table0: [u8; 0x0400],
    name_table1: [u8; 0x0400],
    name_table2: [u8; 0x0400],
    name_table3: [u8; 0x0400],

    pattern_table0: [u8; 0x1000],
    pattern_table1: [u8; 0x1000],

    palette: [u8; 0x0020],
    // colors: Palette,
    buffer: u8,
    mapper: Rc<RefCell<dyn Mapper>>,
    cycles: u64,
    curr_scanline: u16,
    nmi_generated: bool,
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>) -> PPU {
        PPU {
            addr: Address::new(),
            ctrl: Control::new(),
            status: Status::new(),
            mask: Mask::new(),
            name_table0: [0; 0x0400],
            name_table1: [0; 0x0400],
            name_table2: [0; 0x0400],
            name_table3: [0; 0x0400],
            pattern_table0: mapper.borrow().get_chr_rom()[0x0000..0x1000]
                .try_into()
                .unwrap(),
            pattern_table1: mapper.borrow().get_chr_rom()[0x1000..0x2000]
                .try_into()
                .unwrap(),
            palette: [0; 0x0020],
            // colors: Palette::default(),
            mapper: mapper.clone(),
            buffer: 0,
            cycles: 0,
            curr_scanline: 0,
            nmi_generated: false,
        }
    }

    pub fn render(&self) -> Frame {
        let mut frame = Frame::new();
        let pattern_table =
            self.get_pattern_table_from_bool(self.ctrl.contains(Control::BACKGROUND_PATTERN_ADDR));
        let name_table = self.get_name_table_from_idx(
            self.ctrl.contains(Control::NAMETABLE_2),
            self.ctrl.contains(Control::NAMETABLE_1),
        );
        let tile_size = 8;
        let tiles_per_row = 32;

        // Each bank has 256 tiles
        (0usize..960).for_each(|n| {
            let t = name_table[n] as usize;
            let x = (n % tiles_per_row) * tile_size;
            let y = (n / tiles_per_row) * tile_size;
            let range = (t % 256) * 16..((t % 256) * 16 + 16);
            let tile = &pattern_table[range];
            (0..8).cartesian_product(0..8).for_each(|(py, px)| {
                let upper = tile[py];
                let lower = tile[py + 8];

                let mask = 7 - px;
                let bit1 = 1 & (upper >> mask);
                let bit0 = 1 & (lower >> mask);
                let val = (bit1 << 1) | bit0;

                let color = self.get_color(val);
                frame.set_pixel(x + px, y + py, color);
            });
        });

        frame
    }

    pub fn render_tiles(&self) -> Frame {
        // Each tile is 16 bytes
        let mut frame = Frame::new();
        let tile_gap = 2;
        let tile_size = 8 + tile_gap;
        let tiles_per_row = 25;

        // Each bank has 256 tiles
        (0usize..512).for_each(|n| {
            let x = (n % tiles_per_row) * tile_size;
            let y = (n / tiles_per_row) * tile_size;
            let range = (n % 256) * 16..((n % 256) * 16 + 16);
            let tile = if n < 256 {
                &self.pattern_table0[range]
            } else {
                &self.pattern_table1[range]
            };
            (0..8).cartesian_product(0..8).for_each(|(py, px)| {
                let lower = tile[py];
                let upper = tile[py + 8];

                let mask = 7 - px;
                let bit1 = 1 & (upper >> mask);
                let bit0 = 1 & (lower >> mask);
                let val = (bit1 << 1) | bit0;

                let color = self.get_color(val);
                frame.set_pixel(x + px, y + py, color);
            });
        });
        frame
    }

    // Get NMI *and* reset it
    pub fn poll_nmi(&mut self) -> bool {
        let nmi = self.nmi_generated;
        self.nmi_generated = false;
        nmi
    }

    // Return true for frame completion
    pub fn tick(&mut self, cycles: u64) -> bool {
        self.cycles += cycles;
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.curr_scanline += 1;
            if self.curr_scanline == 241 {
                self.status.set(Status::VBLANK, true);
                self.status.set(Status::SPRITE_ZERO_HIT, false);
                if self.ctrl.contains(Control::NMI) {
                    self.nmi_generated = true;
                }
            } else if self.curr_scanline >= 262 {
                self.curr_scanline = 0;
                self.status.set(Status::VBLANK, false);
                self.status.set(Status::SPRITE_ZERO_HIT, false);
                self.nmi_generated = false;
                return true;
            }
        }
        false
    }

    pub fn set_chr_rom(&mut self, chr_rom: [u8; 0x2000]) {
        self.pattern_table0 = chr_rom[0x0000..0x1000].try_into().unwrap();
        self.pattern_table1 = chr_rom[0x1000..0x2000].try_into().unwrap();
    }

    pub fn write_ppumask(&mut self, val: u8) {
        self.mask.write(val);
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        self.addr.write(val);
    }

    pub fn write_ppuctrl(&mut self, val: u8) {
        let before = self.ctrl;
        self.ctrl.write(val);
        if before.contains(Control::NMI)
            && self.ctrl.contains(Control::NMI)
            && self.status.contains(Status::VBLANK)
        {
            self.nmi_generated = true;
        }
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let addr = self.addr.read() as usize;
        self.increment_addr();
        match addr {
            0x0000..=0x0fff => {
                let res = self.buffer;
                self.buffer = self.pattern_table0[addr];
                res
            }
            0x1000..=0x1fff => {
                let res = self.buffer;
                self.buffer = self.pattern_table1[addr - 0x1000];
                res
            }
            0x2000..=0x3eff => {
                let res = self.buffer;
                self.buffer = self.get_from_nametable(addr);
                res
            }
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f0c => self.palette[addr - 0x3f10],
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) % 0x20],
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    pub fn read_ppustatus(&self) -> u8 {
        self.status.bits()
    }

    pub fn write_ppudata(&mut self, data: u8) {
        let addr = self.addr.read() as usize;
        self.increment_addr();
        match addr {
            0x0000..=0x0fff => self.pattern_table0[addr] = data,
            0x1000..=0x1fff => self.pattern_table1[addr - 0x1000] = data,
            0x2000..=0x3eff => self.write_to_nametable(addr, data),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[addr - 0x3f10] = data,
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) % 0x20] = data,
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    fn write_to_nametable(&mut self, addr: usize, data: u8) {
        let mirroring = self.mapper.borrow().get_mirroring();
        match mirroring {
            Mirroring::FourScreen => match addr {
                0x2000..=0x23ff => self.name_table0[addr - 0x2000] = data,
                0x2400..=0x27ff => self.name_table1[addr - 0x2400] = data,
                0x2800..=0x2bff => self.name_table2[addr - 0x2800] = data,
                0x2c00..=0x2fff => self.name_table3[addr - 0x2c00] = data,
                0x3000..=0x3eff => self.write_to_nametable(addr - 0x1000, data),
                _ => panic!("Invalid address {:#X}", addr),
            },
            Mirroring::Horizontal => match addr {
                0x2000..=0x23ff => {
                    self.name_table0[addr - 0x2000] = data;
                    self.name_table1[addr - 0x2000] = data;
                }
                0x2400..=0x27ff => {
                    self.name_table0[addr - 0x2400] = data;
                    self.name_table1[addr - 0x2400] = data;
                }
                0x2800..=0x2bff => {
                    self.name_table2[addr - 0x2800] = data;
                    self.name_table3[addr - 0x2800] = data;
                }
                0x2c00..=0x2fff => {
                    self.name_table2[addr - 0x2c00] = data;
                    self.name_table3[addr - 0x2c00] = data;
                }
                0x3000..=0x3eff => self.write_to_nametable(addr - 0x1000, data),
                _ => panic!("Invalid address {:#X}", addr),
            },
            Mirroring::Vertical => match addr {
                0x2000..=0x23ff => {
                    self.name_table0[addr - 0x2000] = data;
                    self.name_table2[addr - 0x2000] = data;
                }
                0x2400..=0x27ff => {
                    self.name_table1[addr - 0x2400] = data;
                    self.name_table3[addr - 0x2400] = data;
                }
                0x2800..=0x2bff => {
                    self.name_table0[addr - 0x2800] = data;
                    self.name_table2[addr - 0x2800] = data;
                }
                0x2c00..=0x2fff => {
                    self.name_table1[addr - 0x2c00] = data;
                    self.name_table3[addr - 0x2c00] = data;
                }
                0x3000..=0x3eff => self.write_to_nametable(addr - 0x1000, data),
                _ => panic!("Invalid address {:#X}", addr),
            },
        }
    }

    fn increment_addr(&mut self) {
        self.addr
            .increment(if self.ctrl.contains(Control::INCREMENT) {
                32
            } else {
                1
            });
    }

    fn get_color(&self, val: u8) -> Color {
        match val {
            0 => Color::RGB(0, 0, 0),
            1 => Color::RGB(102, 102, 102),
            2 => Color::RGB(187, 187, 187),
            3 => Color::RGB(255, 255, 255),
            _ => unreachable!(),
        }
    }

    fn get_from_nametable(&self, addr: usize) -> u8 {
        match addr {
            0x2000..=0x23ff => self.name_table0[addr - 0x2000],
            0x2400..=0x27ff => self.name_table1[addr - 0x2400],
            0x2800..=0x2bff => self.name_table2[addr - 0x2800],
            0x2c00..=0x2fff => self.name_table3[addr - 0x2c00],
            0x3000..=0x3eff => self.get_from_nametable(addr - 0x1000),
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    fn get_pattern_table_from_bool(&self, b: bool) -> &[u8; 4096] {
        if b {
            &self.pattern_table1
        } else {
            &self.pattern_table0
        }
    }

    fn get_name_table_from_idx(&self, b1: bool, b0: bool) -> &[u8; 1024] {
        match (b1, b0) {
            (false, false) => &self.name_table0,
            (false, true) => &self.name_table1,
            (true, false) => &self.name_table2,
            (true, true) => &self.name_table3,
        }
    }
}
