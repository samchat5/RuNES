use std::sync::{Arc, Mutex};

use crate::ines_parser::{File, Flags1Enum};

use self::{cnrom::CNROM, mmc1::MMC1, nrom::NROM};

pub mod cnrom;
pub mod mmc1;
pub mod nrom;

pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
    SingleScreenA,
    SingleScreenB,
}

pub type SharedMapper = Arc<Mutex<Box<dyn Mapper + Send>>>;

pub struct MapperFactory;

macro_rules! mappers {
    ($file:expr, $( ($num:pat, $to_create:ty) ),*) => {
        {
            let mapper_num = $file.header.flags1.get(Flags1Enum::MAPPER_NUM);
            let prg_ram_size = $file.get_prg_ram_size();
            let eeprom_size = $file.get_eeprom_size();
            let has_battery = $file.header.flags1.get(Flags1Enum::BATTERY) != 0;
            let prg_rom_area = $file.prg_rom_area.clone();
            let chr_rom_area = $file.chr_rom_area.clone();
            let mirroring = $file.header.flags1.get(Flags1Enum::NAME_TABLE_MIRROR);

            match mapper_num {
                $(
                    $num => Box::new(<$to_create>::new(
                        prg_rom_area,
                        chr_rom_area,
                        prg_ram_size,
                        eeprom_size,
                        has_battery,
                        mirroring,
                    )),
                )*
                _ => panic!("Unsupported mapper {}", mapper_num)
            }
        }
    };
}

impl MapperFactory {
    pub fn from_file(file: &File) -> Box<dyn Mapper + Send> {
        mappers!(file, (0, NROM), (1, MMC1), (3, CNROM))
    }
}

pub trait Mapper {
    fn get_mirroring(&self) -> Mirroring;

    fn read_chr_rom(&self, addr: u16) -> u8;

    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);

    fn write_chr_rom(&mut self, _addr: u16, _data: u8) {}

    fn write_nametable_idx(&mut self, idx: usize, addr: u16, val: u8);

    fn read_nametable_idx(&self, idx: usize, addr: u16) -> u8;

    fn read_16(&self, addr: u16) -> u16 {
        let low = self.read(addr);
        let high = self.read(addr + 1);
        (high as u16) << 8 | low as u16
    }

    fn write_16(&mut self, addr: u16, data: u16) {
        self.write(addr, data as u8);
        self.write(addr + 1, (data >> 8) as u8);
    }

    fn read_trace(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn read_16_trace(&self, addr: u16) -> u16 {
        let low = self.read_trace(addr);
        let high = self.read_trace(addr + 1);
        (high as u16) << 8 | low as u16
    }

    fn get_nametable_idx(&self, i: u8) -> usize {
        let mapping = match self.get_mirroring() {
            Mirroring::Horizontal => [0, 0, 1, 1],
            Mirroring::Vertical => [0, 1, 0, 1],
            Mirroring::FourScreen => [0, 1, 2, 3],
            Mirroring::SingleScreenA => [0, 0, 0, 0],
            Mirroring::SingleScreenB => [1, 1, 1, 1],
        };
        mapping[i as usize]
    }

    fn write_nametable(&mut self, addr: u16, val: u8) {
        match addr {
            0x2000..=0x2FFF => {
                let idx = (addr - 0x2000) / 0x400;
                self.write_nametable_idx(self.get_nametable_idx(idx as u8), addr % 0x400, val);
            }
            0x3000..=0x3EFF => self.write_nametable(addr - 0x1000, val),
            _ => panic!("Invalid address {:#X}", addr),
        }
    }

    fn read_nametable(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x2FFF => {
                let idx = (addr - 0x2000) / 0x400;
                self.read_nametable_idx(self.get_nametable_idx(idx as u8), addr % 0x400)
            }
            _ => panic!("Invalid address {:#X}", addr),
        }
    }
}
