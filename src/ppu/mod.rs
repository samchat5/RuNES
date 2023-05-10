use std::cell::RefCell;
use std::rc::Rc;

use crate::ppu::palettes::Palette;
use crate::{
    frame::Frame,
    mappers::{Mapper, Mirroring},
};

use self::registers::{control::Control, mask::Mask, status::Status};

pub mod palettes;
mod registers;

pub enum DMAFlag {
    Enabled(u8),
    Disabled,
}

#[derive(Debug, Default, Clone, Copy)]
struct Tile {
    palette_offset: u32,
    tile_addr: u16,
    low: u8,
    high: u8,
    offset_y: u8,
}

#[derive(Debug, Default, Clone, Copy)]
struct Sprite {
    offset_y: u8,
    tile_addr: u16,
    palette_offset: u32,
    priority: bool,
    flip_horizontal: bool,
    flip_vertical: bool,
    sprite_x: u8,
    low_byte: u8,
    high_byte: u8,
}

pub struct PPU {
    // PPU Registers
    ctrl: Control,
    status_flags: Status,
    status: u8,
    mask: Mask,

    // Contains all 64 sprites in OAM
    sprite_ram_addr: u32,
    sprite_ram: [u8; 0x100],
    // Contains the 8 sprites that will be drawn on the next scanline
    sprite_tiles: [Sprite; 8],

    // Name tables
    name_table0: [u8; 0x0400],
    name_table1: [u8; 0x0400],
    name_table2: [u8; 0x0400],
    name_table3: [u8; 0x0400],

    pub(crate) cycle: u64,
    pub(crate) scanline: i16,
    palette: [u8; 0x0020],
    colors: Palette,
    pub curr_frame: Frame,

    pub nmi_generated: bool,
    mapper: Rc<RefCell<dyn Mapper>>,

    // Represents the first cycle a BG pixel or sprite can be draw. Modified by mask and enable
    // flags, but is otherwise 0
    minimum_draw_bg_cycle: u32,
    minimum_draw_sprite_cycle: u32,

    // Tile registers. At the beginning of each scanline, the PPU loads the first 2 tiles of the
    // next scanline into these registers
    high_bit_shift: u16,
    low_bit_shift: u16,

    // Buffer containing info on if the dot on this scanline contains a sprite. Cycles 0-256 involve
    // OAM read and sprite eval, before sprite fetches for next scanline
    has_sprite: [bool; 257],

    sprite_count: u8,

    // Buffers for current/last/nest tiles and sprites
    previous_tile: Tile,
    current_tile: Tile,
    next_tile: Tile,

    // PPU internal registers https://www.nesdev.org/wiki/PPU_scrolling#PPU_internal_registers
    vram_addr: u16,
    // t, represents addr of top left onscreen tile
    temp_vram_addr: u16,
    x_scroll: u8,
    w: bool, // w

    sprite_index: u8,
    oam_copy_buffer: u8,
    secondary_sprite_ram: [u8; 32],
    sprite_0_added: bool,
    sprite_in_range: bool,
    secondary_oam_addr: u32,
    overflow_bug_counter: u8,
    oam_copy_done: bool,
    sprite_addr_h: u8,
    sprite_addr_l: u8,
    first_visible_sprite_addr: u8,
    last_visible_sprite_addr: u8,
    sprite_0_visible: bool,
    frame_count: usize,
    prev_rendering_enabled: bool,
    rendering_enabled: bool,
    need_state_update: bool,
    prevent_vbl_flag: bool,
    memory_read_buffer: u8,
    ppu_bus_address: u16,
    update_vram_addr_delay: u8,
    update_vram_addr: u16,
    master_clock: u64,
    pub open_bus: u8,
    pub sprite_dma_transfer: DMAFlag,
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>) -> PPU {
        PPU {
            ctrl: Control::new(),
            status_flags: Status::new(),
            status: 0,
            mask: Mask::new(),
            sprite_ram_addr: 0,
            sprite_ram: [0; 0x100],
            sprite_tiles: [Sprite::default(); 8],
            name_table0: [0; 0x0400],
            name_table1: [0; 0x0400],
            name_table2: [0; 0x0400],
            name_table3: [0; 0x0400],
            palette: [
                0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00,
                0x04, 0x2C, 0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02,
                0x00, 0x20, 0x2C, 0x08,
            ],
            colors: Palette::default(),
            cycle: 0,
            scanline: 0,
            curr_frame: Frame::new(),
            nmi_generated: false,
            mapper,
            minimum_draw_bg_cycle: 0,
            minimum_draw_sprite_cycle: 0,
            high_bit_shift: 0,
            low_bit_shift: 0,
            has_sprite: [false; 257],
            sprite_count: 0,
            previous_tile: Tile::default(),
            current_tile: Tile::default(),
            next_tile: Tile::default(),
            vram_addr: 0,
            temp_vram_addr: 0,
            x_scroll: 0,
            w: false,
            sprite_index: 0,
            oam_copy_buffer: 0,
            secondary_sprite_ram: [0; 32],
            sprite_0_added: false,
            sprite_in_range: false,
            secondary_oam_addr: 0,
            overflow_bug_counter: 0,
            oam_copy_done: false,
            sprite_addr_h: 0,
            sprite_addr_l: 0,
            first_visible_sprite_addr: 0,
            last_visible_sprite_addr: 0,
            sprite_0_visible: false,
            frame_count: 1,
            prev_rendering_enabled: false,
            rendering_enabled: false,
            need_state_update: false,
            prevent_vbl_flag: false,
            memory_read_buffer: 0,
            ppu_bus_address: 0,
            update_vram_addr_delay: 0,
            update_vram_addr: 0,
            master_clock: 0,
            open_bus: 0,
            sprite_dma_transfer: DMAFlag::Disabled,
        }
    }

    fn update_video_ram_addr(&mut self) {
        if self.scanline >= 240 || !self.is_rendering_enabled() {
            self.vram_addr = (self.vram_addr
                + (if self.ctrl.contains(Control::INCREMENT) {
                    32
                } else {
                    1
                }))
                & 0x7FFF;
            self.set_bus_address(self.vram_addr & 0x3fff);
        } else {
            self.increment_scroll_x();
            self.increment_scroll_y();
        }
    }

    fn set_bus_address(&mut self, addr: u16) {
        self.ppu_bus_address = addr;
    }

    fn is_rendering_enabled(&self) -> bool {
        self.rendering_enabled
    }

    fn load_tile_info(&mut self) {
        if self.is_rendering_enabled() {
            // First 240 scanlines run on 8 cycle intervals for tile loading, we'll emulate the
            // fetch on the first cycle of a memory access
            //
            // Cycles 1-2: Fetch from nametable
            // Cycles 3-4: Fetch from attribute table
            // Cycles 5-6: Fetch pattern table low
            // Cycles 7-8/0: Fetch pattern table high
            match self.cycle & 0x07 {
                1 => {
                    self.previous_tile = self.current_tile;
                    self.current_tile = self.next_tile;

                    self.low_bit_shift |= self.next_tile.low as u16;
                    self.high_bit_shift |= self.next_tile.high as u16;

                    let tile_index = self.read_vram(self.get_nametable_addr()) as u16;
                    let tile_addr = (tile_index << 4)
                        | (self.vram_addr >> 12)
                        | self.get_background_pattern_addr();
                    self.next_tile.tile_addr = tile_addr;
                    self.next_tile.offset_y = (self.vram_addr >> 12) as u8;
                }
                3 => {
                    // ORs 2nd bits of coarse x and y scrolls -> YX0
                    let shift = ((self.vram_addr >> 4) & 0x04) | (self.vram_addr & 0x02);
                    self.next_tile.palette_offset =
                        (((self.read_vram(self.get_attribute_addr()) >> shift) & 0x03) << 2) as u32;
                }
                5 => {
                    self.next_tile.low = self.read_vram(self.next_tile.tile_addr);
                }
                7 => {
                    self.next_tile.high = self.read_vram(self.next_tile.tile_addr + 8);
                }
                _ => {}
            }
        }
    }

    fn get_background_pattern_addr(&self) -> u16 {
        if self.ctrl.contains(Control::BACKGROUND_PATTERN_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_attribute_addr(&self) -> u16 {
        0x23C0
            | (self.vram_addr & 0x0C00)
            | ((self.vram_addr >> 4) & 0x38)
            | ((self.vram_addr >> 2) & 0x07)
    }

    fn get_nametable_addr(&self) -> u16 {
        0x2000 | (self.vram_addr & 0x0FFF)
    }

    fn update_minimum_draw_cycles(&mut self) {
        self.minimum_draw_bg_cycle = match (
            self.mask.contains(Mask::SHOW_BACKGROUND),
            self.mask.contains(Mask::SHOW_LEFT_BACKGROUND),
        ) {
            (true, true) => 0,
            (true, false) => 8,
            (false, _) => 300,
        };
        self.minimum_draw_sprite_cycle = match (
            self.mask.contains(Mask::SHOW_SPRITES),
            self.mask.contains(Mask::SHOW_LEFT_SPRITES),
        ) {
            (true, true) => 0,
            (true, false) => 8,
            (false, _) => 300,
        };
    }

    pub fn run_to(&mut self, cycle: u64) -> bool {
        let mut new_frame = false;
        while self.master_clock + 4 <= cycle {
            new_frame |= self.run();
            self.master_clock += 4;
        }
        new_frame
    }

    fn run(&mut self) -> bool {
        if self.cycle > 339 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 260 {
                self.scanline = -1;
                self.sprite_count = 0;
                self.update_minimum_draw_cycles();
            }
            if self.scanline == -1 {
                self.status_flags.set(Status::SPRITE_OVERFLOW, false);
                self.status_flags.set(Status::SPRITE_ZERO_HIT, false);
                self.curr_frame = Frame::new();
            } else if self.scanline == 240 {
                self.set_bus_address(self.vram_addr);
                self.frame_count += 1;
                return true;
            }
        } else {
            self.cycle += 1;
            if self.scanline < 240 {
                self.process_scanline();
            } else if self.cycle == 1 && self.scanline == 241 {
                if !self.prevent_vbl_flag {
                    self.status_flags.set(Status::VBLANK, true);
                    if self.ctrl.contains(Control::NMI) {
                        self.nmi_generated = true;
                    }
                }
                self.prevent_vbl_flag = false;
            }
        }

        if self.need_state_update {
            self.update_state();
        }

        false
    }

    fn load_sprite_tile_info(&mut self) {
        let sprite_addr = self.sprite_index as u16 * 4;
        self.load_sprite(sprite_addr as usize);
    }

    fn load_sprite(&mut self, sprite_addr: usize) {
        let data: &[u8] = &self.secondary_sprite_ram[sprite_addr..sprite_addr + 4];
        let sprite_y = data[0];
        let tile_idx = data[1];
        let attr = data[2];
        let sprite_x = data[3];

        let background_priority = attr & 0x20 == 0x20;
        let horizontal_mirror = attr & 0x40 == 0x40;
        let vertical_mirror = attr & 0x80 == 0x80;

        let line_offset = if vertical_mirror {
            ((if self.ctrl.contains(Control::SPRITE_SIZE) {
                15
            } else {
                7
            }) - (self.scanline - sprite_y as i16)) as u8
        } else {
            (self.scanline - sprite_y as i16) as u8
        };

        let tile_addr = if self.ctrl.contains(Control::SPRITE_SIZE) {
            let tile_addr_1 =
                ((tile_idx as u16 & 0x01) * 0x1000) | ((tile_idx as u16 & !0x01) << 4);
            let tile_addr_2 = (if line_offset >= 8 {
                line_offset.wrapping_add(8)
            } else {
                line_offset
            }) as u16;
            tile_addr_1 + tile_addr_2
        } else {
            ((tile_idx as u16) << 4)
                | ((if self.ctrl.contains(Control::SPRITE_PATTERN_ADDR) {
                    0x1000
                } else {
                    0x0000
                }) + line_offset as u16)
        };

        if self.sprite_index < self.sprite_count && sprite_y < 240 {
            let low_byte = self.read_vram(tile_addr);
            let high_byte = self.read_vram(tile_addr + 8);
            let info = &mut self.sprite_tiles[self.sprite_index as usize];
            info.priority = background_priority;
            info.flip_horizontal = horizontal_mirror;
            info.flip_vertical = vertical_mirror;
            info.palette_offset = (((attr & 0x03) << 2) | 0x10) as u32;
            info.low_byte = low_byte;
            info.high_byte = high_byte;
            info.tile_addr = tile_addr;
            info.offset_y = line_offset;
            info.sprite_x = sprite_x;
            if self.scanline >= 0 {
                let mut i = 0;
                while i < 8 && (sprite_x as u16 + i + 1) < 257 {
                    self.has_sprite[(sprite_x as u16 + i + 1) as usize] = true;
                    i += 1;
                }
            }
        }

        self.sprite_index += 1;
    }

    fn draw_pixel(&mut self) {
        if self.is_rendering_enabled() || ((self.vram_addr & 0x3f00) != 0x3f00) {
            let pixel_color = self.get_pixel_color();
            let color = self.colors.system_palette[(self.palette[(if pixel_color & 0x03 > 0 {
                pixel_color
            } else {
                0
            }) as usize]) as usize];
            self.curr_frame
                .set_pixel((self.cycle - 1) as usize, self.scanline as usize, color);
        } else {
            self.curr_frame.set_pixel(
                (self.cycle - 1) as usize,
                self.scanline as usize,
                self.colors.system_palette[self.palette[(self.vram_addr & 0x1f) as usize] as usize],
            );
        }
    }

    fn shift_tile_registers(&mut self) {
        self.low_bit_shift <<= 1;
        self.high_bit_shift <<= 1;
    }

    fn read_sprite_ram(&self, addr: usize) -> u8 {
        self.sprite_ram[addr]
    }

    fn process_sprite_eval(&mut self) {
        if self.is_rendering_enabled() {
            if self.cycle < 65 {
                self.oam_copy_buffer = 0xff;
                self.secondary_sprite_ram[((self.cycle - 1) >> 1) as usize] = 0xff;
            } else {
                if self.cycle == 65 {
                    self.sprite_0_added = false;
                    self.sprite_in_range = false;
                    self.secondary_oam_addr = 0;

                    self.overflow_bug_counter = 0;

                    self.oam_copy_done = false;
                    self.sprite_addr_h = ((self.sprite_ram_addr >> 2) & 0x3f) as u8;
                    self.sprite_addr_l = (self.sprite_ram_addr & 0x03) as u8;

                    self.first_visible_sprite_addr = self.sprite_addr_h * 4;
                    self.last_visible_sprite_addr = self.first_visible_sprite_addr;
                } else if self.cycle == 256 {
                    self.sprite_0_visible = self.sprite_0_added;
                    self.sprite_count = (self.secondary_oam_addr >> 2) as u8;
                }

                if self.cycle & 0x01 > 0 {
                    self.oam_copy_buffer = self.read_sprite_ram(self.sprite_ram_addr as usize);
                } else {
                    if self.oam_copy_done {
                        self.sprite_addr_h = (self.sprite_addr_h + 1) & 0x3f;
                        if self.secondary_oam_addr >= 0x20 {
                            self.oam_copy_buffer = self.secondary_sprite_ram
                                [(self.secondary_oam_addr & 0x1f) as usize];
                        }
                    } else {
                        if !self.sprite_in_range
                            && self.scanline >= self.oam_copy_buffer as i16
                            && self.scanline
                                < (self.oam_copy_buffer
                                    + if self.ctrl.contains(Control::SPRITE_SIZE) {
                                        16
                                    } else {
                                        8
                                    }) as i16
                        {
                            self.sprite_in_range = true;
                        }

                        if self.secondary_oam_addr < 0x20 {
                            self.secondary_sprite_ram[self.secondary_oam_addr as usize] =
                                self.oam_copy_buffer;

                            if self.sprite_in_range {
                                self.sprite_addr_l += 1;
                                self.secondary_oam_addr += 1;

                                if self.sprite_addr_h == 0 {
                                    self.sprite_0_added = true;
                                }

                                if (self.secondary_oam_addr & 0x03) == 0 {
                                    self.sprite_in_range = false;
                                    self.sprite_addr_l = 0;
                                    self.last_visible_sprite_addr = self.sprite_addr_h * 4;
                                    self.sprite_addr_h = (self.sprite_addr_h + 1) & 0x3f;
                                    if self.sprite_addr_h == 0 {
                                        self.oam_copy_done = true;
                                    }
                                }
                            } else {
                                self.sprite_addr_h = (self.sprite_addr_h + 1) & 0x3f;
                                if self.sprite_addr_h == 0 {
                                    self.oam_copy_done = true;
                                }
                            }
                        } else {
                            self.oam_copy_buffer = self.secondary_sprite_ram
                                [(self.secondary_oam_addr & 0x1f) as usize];

                            if self.sprite_in_range {
                                self.status_flags.set(Status::SPRITE_OVERFLOW, true);
                                self.sprite_addr_l += 1;
                                if self.sprite_addr_l == 4 {
                                    self.sprite_addr_h = (self.sprite_addr_h + 1) & 0x3f;
                                    self.sprite_addr_l = 0;
                                }

                                match self.overflow_bug_counter {
                                    0 => {
                                        self.overflow_bug_counter = 3;
                                    }
                                    1 => {
                                        self.overflow_bug_counter = 0;
                                        self.oam_copy_done = true;
                                        self.sprite_addr_l = 0;
                                    }
                                    _ => {
                                        self.overflow_bug_counter -= 1;
                                    }
                                }
                            } else {
                                self.sprite_addr_h = (self.sprite_addr_h + 1) & 0x3f;
                                self.sprite_addr_l = (self.sprite_addr_l + 1) & 0x03;

                                if self.sprite_addr_h == 0 {
                                    self.oam_copy_done = true;
                                }
                            }
                        }
                    }
                    self.sprite_ram_addr =
                        (self.sprite_addr_l as u32 & 0x03) | ((self.sprite_addr_h as u32) << 2);
                }
            }
        }
    }

    fn process_scanline(&mut self) {
        if self.cycle <= 256 {
            self.load_tile_info();

            if self.prev_rendering_enabled && (self.cycle & 0x07) == 0 {
                self.increment_scroll_x();
                if self.cycle == 256 {
                    self.increment_scroll_y();
                }
            }

            if self.scanline >= 0 {
                self.draw_pixel();
                self.shift_tile_registers();
                self.process_sprite_eval();
            } else if self.cycle < 9 {
                if self.cycle == 1 {
                    self.status_flags.set(Status::VBLANK, false);
                    self.nmi_generated = false;
                }
                if self.sprite_ram_addr >= 0x08 && self.is_rendering_enabled() {
                    self.sprite_ram[(self.cycle - 1) as usize] = self.sprite_ram
                        [((self.sprite_ram_addr as u64 & 0xf8) + self.cycle - 1) as usize];
                }
            }
        } else if self.cycle >= 257 && self.cycle <= 320 {
            if self.cycle == 257 {
                self.sprite_index = 0;
                self.has_sprite = [false; 257];
                if self.prev_rendering_enabled {
                    self.vram_addr = (self.vram_addr & !0x041f) | (self.temp_vram_addr & 0x041f);
                }
            }
            if self.is_rendering_enabled() {
                self.sprite_ram_addr = 0;
                if self.cycle.wrapping_sub(261) % 8 == 0 {
                    self.load_sprite_tile_info();
                } else if self.cycle.wrapping_sub(257) % 8 == 0 {
                    // Garbage NT fetch
                    self.read_vram(self.get_nametable_addr());
                } else if self.cycle.wrapping_sub(259) % 8 == 0 {
                    // Garbage AT fetch
                    self.read_vram(self.get_attribute_addr());
                }

                if self.scanline == -1 && self.cycle >= 280 && self.cycle <= 304 {
                    self.vram_addr = (self.vram_addr & !0x7be0) | (self.temp_vram_addr & 0x7be0);
                }
            }
        } else if self.cycle >= 321 && self.cycle <= 336 {
            if self.cycle == 321 {
                if self.is_rendering_enabled() {
                    self.oam_copy_buffer = self.secondary_sprite_ram[0];
                }
                self.load_tile_info();
            } else if self.prev_rendering_enabled && (self.cycle == 328 || self.cycle == 336) {
                self.load_tile_info();
                self.low_bit_shift <<= 8;
                self.high_bit_shift <<= 8;
                self.increment_scroll_x()
            } else {
                self.load_tile_info();
            }
        } else if (self.cycle == 337 || self.cycle == 339) && self.is_rendering_enabled() {
            self.read_vram(self.get_nametable_addr());
            if self.scanline == -1 && self.cycle == 339 && (self.frame_count % 2 == 1) {
                self.cycle = 340;
            }
        }
    }

    fn update_state(&mut self) {
        self.need_state_update = false;
        if self.prev_rendering_enabled != self.rendering_enabled {
            self.prev_rendering_enabled = self.rendering_enabled;
            if self.scanline < 240 && !self.prev_rendering_enabled {
                self.set_bus_address(self.vram_addr & 0x3fff);

                if self.cycle >= 65 && self.cycle <= 256 {
                    self.sprite_ram_addr += 1;
                    self.sprite_addr_h = ((self.sprite_ram_addr >> 2) & 0x3f) as u8;
                    self.sprite_addr_l = (self.sprite_ram_addr & 0x03) as u8;
                }
            }
        }

        if self.rendering_enabled
            != (self.mask.contains(Mask::SHOW_BACKGROUND) || self.mask.contains(Mask::SHOW_SPRITES))
        {
            self.rendering_enabled =
                self.mask.contains(Mask::SHOW_BACKGROUND) || self.mask.contains(Mask::SHOW_SPRITES);
            self.need_state_update = true;
        }

        if self.update_vram_addr_delay > 0 {
            self.update_vram_addr_delay -= 1;
            if self.update_vram_addr_delay == 0 {
                self.vram_addr = self.update_vram_addr;

                self.temp_vram_addr = self.vram_addr;

                if self.scanline >= 240 || !self.is_rendering_enabled() {
                    self.set_bus_address(self.vram_addr & 0x3fff);
                }
            } else {
                self.need_state_update = true;
            }
        }
    }

    fn increment_scroll_x(&mut self) {
        let mut addr = self.vram_addr;
        if (addr & 0x1f) == 31 {
            addr = (addr & !0x1f) ^ 0x400;
        } else {
            addr += 1
        }
        self.vram_addr = addr;
    }

    fn increment_scroll_y(&mut self) {
        let mut addr = self.vram_addr;
        if (addr & 0x7000) != 0x7000 {
            addr += 0x1000;
        } else {
            addr &= !0x7000;
            let mut y = (addr & 0x03e0) >> 5;
            if y == 29 {
                y = 0;
                addr ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            addr = (addr & !0x03e0) | (y << 5);
        }
        self.vram_addr = addr;
    }

    pub fn write_ppumask(&mut self, val: u8) {
        self.mask.write(val);
        if self.rendering_enabled
            != (self.mask.contains(Mask::SHOW_BACKGROUND) || self.mask.contains(Mask::SHOW_SPRITES))
        {
            self.need_state_update = true;
        }
        self.update_minimum_draw_cycles();
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        if self.w {
            self.temp_vram_addr = (self.temp_vram_addr & !0x00ff) | val as u16;

            self.need_state_update = true;
            self.update_vram_addr_delay = 3;
            self.update_vram_addr = self.temp_vram_addr;
        } else {
            self.temp_vram_addr = (self.temp_vram_addr & !0xff00) | ((val as u16 & 0x3f) << 8);
        }
        self.w = !self.w;
    }

    pub fn write_ppuctrl(&mut self, val: u8) {
        self.ctrl.write(val);
        let name_table = self.ctrl.bits() & 0x03;
        self.temp_vram_addr = self.temp_vram_addr & !0x0c00 | (name_table as u16) << 10;
        if !self.ctrl.contains(Control::NMI) {
            self.nmi_generated = false;
        } else if self.ctrl.contains(Control::NMI) && self.status_flags.contains(Status::VBLANK) {
            self.nmi_generated = true;
        }
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let mut return_value = self.memory_read_buffer;
        let mut open_bus_mask = 0x00;
        self.memory_read_buffer = self.read_vram(self.ppu_bus_address & 0x3fff);

        if (self.ppu_bus_address & 0x3fff) >= 0x3f00 {
            return_value = self.read_palette_ram(self.ppu_bus_address) | self.open_bus & 0xc0;
            open_bus_mask = 0xc0;
        }

        self.update_video_ram_addr();
        self.need_state_update = true;

        return_value | (self.open_bus & open_bus_mask)
    }

    fn read_palette_ram(&self, addr: u16) -> u8 {
        let mut addr = addr & 0x1f;
        if vec![0x10, 0x14, 0x18, 0x1c].contains(&addr) {
            addr &= !0x10;
        }
        self.palette[addr as usize]
    }

    pub fn read_ppudata_trace(&self, addr: usize) -> u8 {
        match addr {
            0x0000..=0x1fff => self.mapper.borrow().read_chr_rom(addr as u16),
            0x2000 => self.ctrl.bits(),
            0x2001 => self.mask.bits(),
            0x2002 => self.status,
            0x2003 => self.sprite_ram_addr as u8,
            0x2004 => self.sprite_ram[self.sprite_ram_addr as usize],
            0x2005 => self.x_scroll,
            0x2006 => self.temp_vram_addr as u8,
            0x2007 => self.memory_read_buffer,
            0x2008..=0x23ff => self.name_table0[(addr - 0x2000) as usize],
            0x2400..=0x27ff => self.name_table1[(addr - 0x2400) as usize],
            0x2800..=0x2bff => self.name_table2[(addr - 0x2800) as usize],
            0x2c00..=0x2fff => self.name_table3[(addr - 0x2c00) as usize],
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[addr - 0x3f10],
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) % 0x20],
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    fn get_pixel_color(&mut self) -> u8 {
        let offset = self.x_scroll;
        let mut background_color = 0u8;
        let mut sprite_bg_color = 0u8;

        if self.cycle > self.minimum_draw_bg_cycle as u64 {
            // TODO: Comments on this
            let or_1 = ((self.low_bit_shift << (offset as u16)) & 0x8000) >> 15;
            let or_2 = ((self.high_bit_shift << (offset as u16)) & 0x8000) >> 14;
            sprite_bg_color = (or_1 | or_2) as u8;
            background_color = sprite_bg_color;
        }
        if self.has_sprite[self.cycle as usize]
            && self.cycle > self.minimum_draw_sprite_cycle as u64
        {
            for i in 0..self.sprite_count {
                let shift = self.cycle as i32 - self.sprite_tiles[i as usize].sprite_x as i32 - 1;
                if (0..8).contains(&shift) {
                    let sprite = self.sprite_tiles[i as usize];
                    let sprite_color = if self.sprite_tiles[i as usize].flip_horizontal {
                        ((sprite.low_byte >> shift) & 0x01)
                            | ((sprite.high_byte >> shift) & 0x01) << 1
                    } else {
                        ((sprite.low_byte << shift) & 0x80) >> 7
                            | ((sprite.high_byte << shift) & 0x80) >> 6
                    };
                    if sprite_color != 0 {
                        if i == 0
                            && sprite_bg_color != 0
                            && self.sprite_0_visible
                            && self.cycle != 256
                            && self.mask.contains(Mask::SHOW_BACKGROUND)
                            && !self.status_flags.contains(Status::SPRITE_ZERO_HIT)
                            && self.cycle > self.minimum_draw_sprite_cycle as u64
                        {
                            self.status_flags.set(Status::SPRITE_ZERO_HIT, true);
                        }
                        if background_color == 0 || !self.sprite_tiles[i as usize].priority {
                            return (sprite.palette_offset + sprite_color as u32) as u8;
                        }
                        break;
                    }
                }
            }
        }
        ((if (offset as u64) + ((self.cycle - 1) & 0x07) < 8 {
            self.previous_tile
        } else {
            self.current_tile
        })
        .palette_offset
            + background_color as u32) as u8
    }

    pub fn write_ppuscroll(&mut self, val: u8) {
        if self.w {
            self.temp_vram_addr = (self.temp_vram_addr & !0x73e0)
                | ((val as u16 & 0xf8) << 2)
                | ((val as u16 & 0x07) << 12);
        } else {
            self.x_scroll = val & 0x07;
            self.temp_vram_addr = (self.temp_vram_addr & !0x001f) | (val >> 3) as u16;
        }
        self.w = !self.w;
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        self.w = false;
        self.update_status_flag();
        self.status
    }

    fn update_status_flag(&mut self) {
        self.status = self.status_flags.bits() & 0xe0;
        self.status_flags.set(Status::VBLANK, false);
        self.nmi_generated = false;
        if self.scanline == 241 && self.cycle == 0 {
            self.prevent_vbl_flag = true;
        }
    }

    pub fn write_ppudata(&mut self, data: u8) {
        if self.ppu_bus_address & 0x3fff >= 0x3f00 {
            self.write_palette_ram(self.ppu_bus_address, data);
        } else if self.scanline >= 240 || !self.is_rendering_enabled() {
            self.write_vram(self.ppu_bus_address & 0x3fff, data);
        } else {
            self.write_vram(
                self.ppu_bus_address & 0x3fff,
                (self.ppu_bus_address & 0xff) as u8,
            );
        }
        self.update_video_ram_addr();
    }

    fn write_vram(&mut self, addr: u16, val: u8) {
        self.set_bus_address(addr);
        match addr {
            0x0000..=0x1fff => self.mapper.borrow_mut().write_chr_rom(addr, val),
            0x2000..=0x3eff => self.write_to_nametable(addr as usize, val),
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    fn write_palette_ram(&mut self, addr: u16, val: u8) {
        let addr = addr & 0x1f;
        let val = val & 0x3f;
        match addr {
            0x00 | 0x04 | 0x08 | 0x0c => {
                self.palette[(addr + 0x10) as usize] = val;
            }
            0x10 | 0x14 | 0x18 | 0x1c => {
                self.palette[(addr - 0x10) as usize] = val;
            }
            _ => {
                self.palette[addr as usize] = val;
            }
        }
    }

    pub fn write_oamaddr(&mut self, val: u8) {
        self.sprite_ram_addr = val as u32;
    }

    pub fn write_oamdata(&mut self, val: u8) {
        let mut val = val;
        if self.scanline >= 240 || !self.is_rendering_enabled() {
            if self.sprite_ram_addr & 0x03 == 0x02 {
                val &= 0xe3;
            }
            self.sprite_ram[self.sprite_ram_addr as usize] = val;
            self.sprite_ram_addr = (self.sprite_ram_addr + 1) & 0xff;
        } else {
            self.sprite_ram_addr = (self.sprite_ram_addr + 4) & 0xff;
        }
    }

    pub fn read_oamdata(&mut self) -> u8 {
        if self.scanline <= 239 && self.is_rendering_enabled() {
            if self.cycle >= 257 && self.cycle <= 320 {
                let step: u8 = (if ((self.cycle.wrapping_sub(257)) % 8) > 3 {
                    3
                } else {
                    self.cycle.wrapping_sub(257) % 8
                }) as u8;
                self.secondary_oam_addr =
                    ((self.cycle.wrapping_sub(257)) / 8 * 4 + (step as u64)) as u32;
            }
            self.oam_copy_buffer
        } else {
            self.read_sprite_ram(self.sprite_ram_addr as usize)
        }
    }

    pub fn write_oamdma(&mut self, data: u8) {
        self.sprite_dma_transfer = DMAFlag::Enabled(data);
    }

    fn write_to_nametable(&mut self, addr: usize, data: u8) {
        let mirroring = self.mapper.borrow().get_mirroring();
        match mirroring {
            Mirroring::SingleScreenA => self.name_table0[addr % 0x400] = data,
            Mirroring::SingleScreenB => self.name_table1[addr % 0x400] = data,
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

    fn read_vram(&mut self, addr: u16) -> u8 {
        self.set_bus_address(addr);
        let mirroring = self.mapper.borrow().get_mirroring();
        match mirroring {
            Mirroring::SingleScreenA => match addr {
                0x0000..=0x1fff => self.mapper.borrow().read_chr_rom(addr),
                0x2000..=0x2fff => self.name_table0[(addr % 0x400) as usize],
                0x3000..=0x3fff => self.read_vram(addr - 0x1000),
                _ => panic!("Invalid address {:#X}", addr),
            },
            Mirroring::SingleScreenB => match addr {
                0x0000..=0x1fff => self.mapper.borrow().read_chr_rom(addr),
                0x2000..=0x2fff => self.name_table1[(addr % 0x400) as usize],
                0x3000..=0x3fff => self.read_vram(addr - 0x1000),
                _ => panic!("Invalid address {:#X}", addr),
            },
            _ => match addr {
                0x0000..=0x1fff => self.mapper.borrow().read_chr_rom(addr as u16),
                0x2000..=0x23ff => self.name_table0[(addr - 0x2000) as usize],
                0x2400..=0x27ff => self.name_table1[(addr - 0x2400) as usize],
                0x2800..=0x2bff => self.name_table2[(addr - 0x2800) as usize],
                0x2c00..=0x2fff => self.name_table3[(addr - 0x2c00) as usize],
                0x3000..=0x3fff => self.read_vram(addr - 0x1000),
                _ => panic!("Invalid address {:#X}", addr),
            },
        }
    }
}
