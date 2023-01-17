use crate::mappers::mapper_0::Mapper0;
use crate::mappers::Mapper;

const RAM_SIZE: usize = 0x0800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_REG_SIZE: usize = 0x0008;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

const APU_IO_SIZE: usize = 0x0020;
const APU_IO_START: u16 = 0x4000;
const APU_IO_END: u16 = 0x401F;

pub struct Bus {
    pub cpu_ram: [u8; RAM_SIZE],
    pub ppu_regs: [u8; PPU_REG_SIZE],
    pub apu_io: [u8; APU_IO_SIZE],
    pub cart: Box<dyn Mapper>,
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    pub fn new() -> Self {
        Self {
            cpu_ram: [0; RAM_SIZE],
            ppu_regs: [0; PPU_REG_SIZE],
            apu_io: [0; APU_IO_SIZE],
            cart: Box::new(Mapper0::new(vec![])),
        }
    }

    pub fn load_prg_rom(&mut self, prg_rom: Vec<u8>) {
        self.cart.load_prg_rom(prg_rom);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.ppu_regs[(addr & 0x0007) as usize],
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize],
            _ => self.cart.read(addr),
        }
    }

    pub fn read_16(&self, addr: u16) -> u16 {
        let low = self.read(addr);
        let high = self.read(addr + 1);
        (high as u16) << 8 | low as u16
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize] = data,
            PPU_REG_START..=PPU_REG_END => self.ppu_regs[(addr & 0x0007) as usize] = data,
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize] = data,
            _ => self.cart.write(addr, data),
        }
    }

    pub fn write_16(&mut self, addr: u16, data: u16) {
        self.write(addr, data as u8);
        self.write(addr + 1, (data >> 8) as u8);
    }
}
