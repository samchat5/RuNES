use crate::{ines_parser::File, mappers::nrom::NROM, ppu::PPU};

const RAM_SIZE: usize = 0x0800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

const APU_IO_SIZE: usize = 0x0020;
const APU_IO_START: u16 = 0x4000;
const APU_IO_END: u16 = 0x401F;

pub struct Bus {
    pub cpu_ram: [u8; RAM_SIZE],
    pub apu_io: [u8; APU_IO_SIZE],
    pub ppu: PPU,
}

impl Bus {
    pub fn new(file: File) -> Self {
        Self {
            cpu_ram: [0; RAM_SIZE],
            apu_io: [0; APU_IO_SIZE],
            ppu: PPU::new(Box::new(NROM::new(
                file.prg_rom_area,
                file.chr_rom_area.unwrap(),
            ))),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_read(addr),
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize],
            _ => unimplemented!(),
        }
    }

    pub fn read_16(&mut self, addr: u16) -> u16 {
        let low = self.read(addr);
        let high = self.read(addr + 1);
        (high as u16) << 8 | low as u16
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize] = data,
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_write(addr, data),
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize] = data,
            _ => unimplemented!(),
        }
    }

    pub fn write_16(&mut self, addr: u16, data: u16) {
        self.write(addr, data as u8);
        self.write(addr + 1, (data >> 8) as u8);
    }

    fn execute_ppu_read(&mut self, addr: u16) -> u8 {
        let mapped_addr = (addr - 0x2000) % 8;
        match mapped_addr {
            0 => panic!("Attempted to read from write-only PPU register 0x2000"),
            1 => panic!("Attempted to read from write-only PPU register 0x2001"),
            2 => unimplemented!(),
            3 => panic!("Attempted to read from write-only PPU register 0x2003"),
            4 => unimplemented!(),
            5 => panic!("Attempted to read from write-only PPU register 0x2005"),
            6 => panic!("Attempted to read from write-only PPU register 0x2006"),
            7 => self.ppu.read_ppudata(),
            _ => unreachable!(),
        }
    }

    fn execute_ppu_write(&mut self, addr: u16, data: u8) {
        let mapped_addr = (addr - 0x2000) % 8;
        match mapped_addr {
            0 => self.ppu.write_ppuctrl(data),
            1 => unimplemented!(),
            2 => panic!("Attempted to write to read-only PPU register 0x2002"),
            3 => unimplemented!(),
            4 => unimplemented!(),
            5 => unimplemented!(),
            6 => self.ppu.write_ppuaddr(data),
            7 => self.ppu.write_ppudata(data),
            _ => unreachable!(),
        }
    }
}
