use super::{Mapper, Mirroring};

#[derive(Clone)]
enum PRGRomMode {
    PRG16k,
    PRG32k,
}

#[derive(Clone)]
pub struct NROM {
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    has_chr_ram: bool,
    prg_rom_mode: PRGRomMode,
    mirroring: u8,
    nametables: [[u8; 0x400]; 2],
}

impl NROM {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Option<Vec<u8>>,
        prg_ram_size: usize,
        eeprom_size: usize,
        has_battery: bool,
        mirroring: u8,
    ) -> Self {
        let mut prg_ram_size = prg_ram_size;
        if prg_ram_size == 0 && eeprom_size > 0 {
            prg_ram_size = eeprom_size;
        } else if has_battery {
            prg_ram_size = 0x2000;
        }

        Self {
            prg_ram: vec![0; prg_ram_size],
            prg_rom_mode: if prg_rom.len() <= 16384 {
                PRGRomMode::PRG16k
            } else {
                PRGRomMode::PRG32k
            },
            prg_rom,
            has_chr_ram: chr_rom.is_none(),
            chr_rom: match chr_rom {
                Some(chr_rom) => chr_rom,
                None => vec![0; 8192],
            },
            mirroring,
            nametables: [[0; 0x400]; 2],
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

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                if !self.prg_ram.is_empty() {
                    self.prg_ram[(addr - 0x6000) as usize]
                } else {
                    println!(
                        "Attempted to read from PRG RAM at {:#X} but no PRG RAM is present",
                        addr
                    );
                    0
                }
            }
            0x8000..=0xBFFF => self.prg_rom[(addr - 0x8000) as usize],
            0xC000..=0xFFFF => match self.prg_rom_mode {
                PRGRomMode::PRG16k => self.prg_rom[(addr - 0xC000) as usize],
                PRGRomMode::PRG32k => self.prg_rom[(addr - 0x8000) as usize],
            },
            _ => {
                println!("Invalid address {:#X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if (0x6000..=0x7FFF).contains(&addr) {
            if !self.prg_ram.is_empty() {
                self.prg_ram[(addr - 0x6000) as usize] = data;
            } else {
                println!(
                    "Attempted to write to PRG RAM at {:#X} but no PRG RAM is present",
                    addr
                );
            }
        }
    }

    fn write_chr_rom(&mut self, addr: u16, data: u8) {
        if self.has_chr_ram {
            self.chr_rom[addr as usize] = data;
        }
    }

    fn write_nametable_idx(&mut self, idx: usize, addr: u16, val: u8) {
        self.nametables[idx][addr as usize] = val;
    }

    fn read_nametable_idx(&self, idx: usize, addr: u16) -> u8 {
        self.nametables[idx][addr as usize]
    }
}
