use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use sdl2::pixels::Color;

use crate::ppu::palettes::Palette;
use crate::ppu::registers::scroll::Scroll;
use crate::{
    frame::Frame,
    mappers::{Mapper, Mirroring},
};

use self::registers::{address::Address, control::Control, mask::Mask, status::Status};

pub mod palettes;
mod registers;

struct BoundingBox {
    x_0: usize,
    y_0: usize,
    x_1: usize,
    y_1: usize,
}

pub struct PPU {
    addr: Address,
    ctrl: Control,
    status: Status,
    mask: Mask,
    scroll: Scroll,

    oam_addr: u8,
    oam: [u8; 0x100],

    name_table0: [u8; 0x0400],
    name_table1: [u8; 0x0400],
    name_table2: [u8; 0x0400],
    name_table3: [u8; 0x0400],

    pattern_table0: [u8; 0x1000],
    pattern_table1: [u8; 0x1000],

    palette: [u8; 0x0020],
    colors: Palette,
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
            scroll: Scroll::default(),
            oam_addr: 0,
            oam: [0; 0x100],
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
            colors: Palette::default(),
            mapper: mapper.clone(),
            buffer: 0,
            cycles: 0,
            curr_scanline: 0,
            nmi_generated: false,
        }
    }

    fn render_nametable(
        &self,
        frame: &mut Frame,
        name_table: &[u8],
        rect: BoundingBox,
        shift_x: isize,
        shift_y: isize,
    ) {
        let pattern_table =
            self.get_pattern_table_from_bool(self.ctrl.contains(Control::BACKGROUND_PATTERN_ADDR));
        let tile_size = 8;
        let tiles_per_row = 32;

        (0usize..0x3c0).for_each(|n| {
            let tile_col = n % tiles_per_row;
            let tile_row = n / tiles_per_row;
            let x = tile_col * tile_size;
            let y = tile_row * tile_size;
            let pal = self.get_attribute_table_idx(tile_row, tile_col, name_table);

            let t = name_table[n] as usize;
            let range = (t % 256) * 16..((t % 256) * 16 + 16);
            let tile = &pattern_table[range];

            (0..8).cartesian_product(0..8).for_each(|(py, px)| {
                let (pixel_x, pixel_y) = (x + px, y + py);
                if pixel_x >= rect.x_0
                    && pixel_x < rect.x_1
                    && pixel_y >= rect.y_0
                    && pixel_y < rect.y_1
                {
                    let lower = tile[py];
                    let upper = tile[py + 8];

                    let mask = 7 - px;
                    let bit1 = 1 & (upper >> mask);
                    let bit0 = 1 & (lower >> mask);
                    let val = (bit1 << 1) | bit0;
                    if val == 0 {
                        frame.is_zero[y + py][x + px] = true;
                    }
                    let color = pal[val as usize];
                    frame.set_pixel(
                        (shift_x + pixel_x as isize) as usize,
                        (shift_y + pixel_y as isize) as usize,
                        color,
                    );
                }
            });
        });
    }

    fn render_sprites(&mut self, pattern_table: bool, frame: &mut Frame) {
        let pattern_table = self.get_pattern_table_from_bool(pattern_table);
        let mut zero_hit = false;
        self.oam.chunks_exact(4).rev().for_each(|c| {
            let tile_y = c[0];
            let tile_idx = c[1];
            let attr = c[2];
            let tile_x = c[3];

            let flip_vert = attr & 0x80 == 0x80;
            let flip_horiz = attr & 0x40 == 0x40;
            let priority = attr & 0x20 == 0x20;

            let range = tile_idx as usize * 16..tile_idx as usize * 16 + 16;
            let tile = &pattern_table[range];
            let pal = self.get_attribute_table_idx_sprite(attr & 0b11);

            (0..8).cartesian_product(0..8).for_each(|(py, px)| {
                let lower = tile[py];
                let upper = tile[py + 8];

                let mask = 7 - px;
                let bit1 = 1 & (upper >> mask);
                let bit0 = 1 & (lower >> mask);
                let val = (bit1 << 1) | bit0;

                let x = tile_x as usize + if flip_horiz { 7 - px } else { px };
                let y = tile_y as usize + if flip_vert { 7 - py } else { py };

                if x < 256 && y < 240 && val != 0x00 && (frame.is_zero[y][x] || !priority) {
                    let color = pal[val as usize];
                    frame.set_pixel(x, y, color);
                }

                if x < 256 && y < 240 && val != 0x00 && !frame.is_zero[y][x] {
                    zero_hit = true;
                }
            });
        });
        if zero_hit {
            self.status.set(Status::SPRITE_ZERO_HIT, true);
        }
    }

    pub fn render(&mut self) -> Frame {
        let mut frame = Frame::new();
        let scroll_x = self.scroll.get_x() as usize;
        let scroll_y = self.scroll.get_y() as usize;

        let name_table_idx = (self.ctrl & (Control::NAMETABLE_2 | Control::NAMETABLE_1)).bits();
        let main_name_table = match name_table_idx {
            0 => &self.name_table0,
            1 => &self.name_table1,
            2 => &self.name_table2,
            3 => &self.name_table3,
            _ => unreachable!(),
        };
        let second_name_table = match (name_table_idx, self.mapper.borrow().get_mirroring()) {
            (2, Mirroring::Horizontal) | (1, Mirroring::Vertical) => &self.name_table0,
            (3, Mirroring::Horizontal) | (0, Mirroring::Vertical) => &self.name_table1,
            (0, Mirroring::Horizontal) | (3, Mirroring::Vertical) => &self.name_table2,
            (1, Mirroring::Horizontal) | (2, Mirroring::Vertical) => &self.name_table3,
            _ => unreachable!(),
        };

        self.render_nametable(
            &mut frame,
            main_name_table,
            BoundingBox {
                x_0: scroll_x,
                y_0: scroll_y,
                x_1: 256,
                y_1: 240,
            },
            -(scroll_x as isize),
            -(scroll_y as isize),
        );
        if scroll_x > 0 {
            self.render_nametable(
                &mut frame,
                second_name_table,
                BoundingBox {
                    x_0: 0,
                    y_0: 0,
                    x_1: scroll_x,
                    y_1: 240,
                },
                (256 - scroll_x) as isize,
                0,
            );
        } else if scroll_y > 0 {
            self.render_nametable(
                &mut frame,
                second_name_table,
                BoundingBox {
                    x_0: 0,
                    y_0: 0,
                    x_1: 256,
                    y_1: scroll_y,
                },
                0,
                (240 - scroll_y) as isize,
            );
        }
        self.render_sprites(self.ctrl.contains(Control::SPRITE_PATTERN_ADDR), &mut frame);
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
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[addr - 0x3f10],
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) % 0x20],
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    pub fn read_ppudata_trace(&self, addr: usize) -> u8 {
        match addr {
            0x0000..=0x0fff => self.pattern_table0[addr],
            0x1000..=0x1fff => self.pattern_table1[addr - 0x1000],
            0x2000..=0x3eff => self.get_from_nametable(addr),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[addr - 0x3f10],
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) % 0x20],
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    pub fn write_ppuscroll(&mut self, data: u8) {
        self.scroll.write(data);
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

    pub fn write_oamaddr(&mut self, val: u8) {
        self.oam_addr = val;
    }

    pub fn write_oamdata(&mut self, val: u8) {
        self.oam[self.oam_addr as usize] = val;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn read_oamdata(&self) -> u8 {
        self.oam[self.oam_addr as usize]
    }

    pub fn write_oamdma(&mut self, data: [u8; 256]) {
        self.oam = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
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

    fn get_attribute_table_idx(&self, row: usize, col: usize, name_table: &[u8]) -> Vec<Color> {
        let attr_table_idx = row / 4 * 8 + col / 4;
        let byte = name_table[0x3c0 + attr_table_idx];
        let palette_idx = match (col % 4 / 2, row % 4 / 2) {
            (0, 0) => byte & 0x3,
            (1, 0) => (byte >> 2) & 0x3,
            (0, 1) => (byte >> 4) & 0x3,
            (1, 1) => (byte >> 6) & 0x3,
            _ => unreachable!(),
        };
        self.get_from_sys_palette((4 * palette_idx + 1) as usize)
    }

    fn get_attribute_table_idx_sprite(&self, palette_idx: u8) -> Vec<Color> {
        self.get_from_sys_palette((4 * palette_idx + 0x11) as usize)
    }

    fn get_from_sys_palette(&self, start: usize) -> Vec<Color> {
        vec![
            self.colors.system_palette[self.palette[0] as usize],
            self.colors.system_palette[self.palette[start] as usize],
            self.colors.system_palette[self.palette[start + 1] as usize],
            self.colors.system_palette[self.palette[start + 2] as usize],
        ]
    }
}
