use crate::mappers::{Mapper, Mirroring};

use self::{address::Address, control::Control};

pub mod address;
pub mod control;

pub struct PPU {
    addr: Address,
    ctrl: Control,

    name_table0: [u8; 0x0400],
    name_table1: [u8; 0x0400],
    name_table2: [u8; 0x0400],
    name_table3: [u8; 0x0400],

    pattern_table0: [u8; 0x1000],
    pattern_table1: [u8; 0x1000],

    palette: [u8; 0x0020],

    buffer: u8,
    mapper: Box<dyn Mapper>,
}

impl PPU {
    pub fn new(mapper: Box<dyn Mapper>) -> PPU {
        PPU {
            addr: Address::new(),
            ctrl: Control::new(),
            name_table0: [0; 0x0400],
            name_table1: [0; 0x0400],
            name_table2: [0; 0x0400],
            name_table3: [0; 0x0400],
            pattern_table0: mapper.get_chr_rom()[0x0000..0x1000].try_into().unwrap(),
            pattern_table1: mapper.get_chr_rom()[0x1000..0x2000].try_into().unwrap(),
            palette: [0; 0x0020],
            mapper,
            buffer: 0,
        }
    }

    pub fn set_chr_rom(&mut self, chr_rom: [u8; 0x2000]) {
        self.pattern_table0 = chr_rom[0x0000..0x1000].try_into().unwrap();
        self.pattern_table1 = chr_rom[0x1000..0x2000].try_into().unwrap();
    }

    fn write_to_nametable(&mut self, addr: usize, data: u8) {
        match self.mapper.get_mirroring() {
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

    fn increment_addr(&mut self) {
        self.addr
            .increment(if self.ctrl.contains(Control::INCREMENT) {
                32
            } else {
                1
            });
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        self.addr.write(val);
    }

    pub fn write_ppuctrl(&mut self, val: u8) {
        self.ctrl.write(val);
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

    pub fn write_ppudata(&mut self, data: u8) {
        let addr = self.addr.read() as usize;
        self.increment_addr();
        match addr {
            0x0000..=0x0fff => self.pattern_table0[addr] = data,
            0x1000..=0x1fff => self.pattern_table1[addr - 0x1000] = data,
            0x2000..=0x3eff => self.write_to_nametable(addr, data),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f0c => self.palette[addr - 0x3f10] = data,
            0x3f00..=0x3fff => self.palette[(addr - 0x3f20) % 0x20] = data,
            _ => panic!("Invalid address {:#X}", addr),
        }
    }
}
