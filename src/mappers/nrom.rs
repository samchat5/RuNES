use super::{Mapper, Mirroring};

#[derive(Clone)]
pub struct NROM {
    pub undefined_area: [u8; 0x3fe0],
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            undefined_area: [0; 0x3fe0],
            prg_rom,
            chr_rom,
        }
    }
}

impl Mapper for NROM {
    fn get_mirroring(&self) -> Mirroring {
        Mirroring::Horizontal
    }

    fn get_chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }

    fn read(&self, addr: u16) -> u8 {
        if addr < 0x7FFF {
            self.undefined_area[(addr - 0x4020) as usize]
        } else if self.prg_rom.len() <= 16384 {
            match addr {
                0x8000..=0xBFFF => self.prg_rom[(addr - 0x8000) as usize],
                0xC000..=0xFFFF => self.prg_rom[(addr - 0xC000) as usize],
                _ => panic!("Invalid address {:#X}", addr),
            }
        } else {
            self.prg_rom[(addr - 0x8000) as usize]
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr < 0x7FFF {
            self.undefined_area[(addr - 0x4020) as usize] = data;
        } else {
            self.prg_rom[(addr - 0xc000) as usize] = data;
            if self.prg_rom.len() > 16384 {
                self.prg_rom[(addr - 0x8000) as usize] = data;
            }
        }
    }
}
