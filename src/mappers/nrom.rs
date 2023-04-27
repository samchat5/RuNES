use super::{Mapper, Mirroring};

#[derive(Clone)]
enum PRGRomMode {
    PRG16k,
    PRG32k,
}

#[derive(Clone)]
pub struct NROM {
    pub undefined_area: [u8; 0x3fe0],
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    prg_rom_mode: PRGRomMode,
    mirroring: u8,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>, mirroring: u8) -> Self {
        Self {
            undefined_area: [0; 0x3fe0],
            prg_rom_mode: if prg_rom.len() <= 16384 {
                PRGRomMode::PRG16k
            } else {
                PRGRomMode::PRG32k
            },
            prg_rom,
            chr_rom: match chr_rom {
                Some(chr_rom) => chr_rom,
                None => vec![0; 8192],
            },
            mirroring,
        }
    }
}

impl Mapper for NROM {
    fn get_mirroring(&self) -> Mirroring {
        match self.mirroring {
            0 => Mirroring::Horizontal,
            1 => Mirroring::Vertical,
            _ => unreachable!(),
        }
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }

    fn write_chr_rom(&mut self, addr: u16, data: u8) {
        self.chr_rom[addr as usize] = data;
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xBFFF => self.prg_rom[(addr - 0x8000) as usize],
            0xC000..=0xFFFF => match self.prg_rom_mode {
                PRGRomMode::PRG16k => self.prg_rom[(addr - 0xC000) as usize],
                PRGRomMode::PRG32k => self.prg_rom[(addr - 0x8000) as usize],
            },
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr < 0x7FFF {
            self.undefined_area[(addr - 0x4020) as usize] = data;
        }
    }
}
