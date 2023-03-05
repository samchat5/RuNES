use std::{cell::RefCell, rc::Rc};

use crate::{
    ines_parser::File,
    mappers::{nrom::NROM, Mapper},
    ppu::PPU,
};

const RAM_SIZE: usize = 0x0800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;
const APU_IO_SIZE: usize = 0x0020;
const APU_IO_START: u16 = 0x4000;
const APU_IO_END: u16 = 0x401F;

pub struct Bus<'a> {
    cpu_ram: [u8; RAM_SIZE],
    apu_io: [u8; APU_IO_SIZE],
    pub ppu: PPU,
    cpu_cycles: u64,
    mapper: Rc<RefCell<dyn Mapper>>,
    callback: Box<dyn FnMut(&PPU) + 'a>,
}

impl<'a> Bus<'a> {
    pub fn new<F>(file: File, callback: F) -> Bus<'a>
    where
        F: FnMut(&PPU) + 'a,
    {
        let mapper = Rc::new(RefCell::new(NROM::new(
            file.prg_rom_area,
            file.chr_rom_area.unwrap(),
        )));
        Bus {
            cpu_ram: [0; RAM_SIZE],
            apu_io: [0; APU_IO_SIZE],
            mapper: mapper.clone(),
            ppu: PPU::new(mapper),
            cpu_cycles: 0,
            callback: Box::new(callback),
        }
    }

    pub fn get_cycles(&self) -> u64 {
        self.cpu_cycles
    }

    pub fn tick(&mut self, cycles: u64) {
        self.cpu_cycles += cycles;
        let new_frame = self.ppu.tick(cycles * 3);
        if new_frame {
            (self.callback)(&self.ppu);
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_read(addr as u16),
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize],
            _ => self.mapper.borrow().read(addr),
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
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_write(addr as u16, data),
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize] = data,
            _ => self.mapper.borrow_mut().write(addr, data),
        }
    }

    pub fn write_16(&mut self, addr: u16, data: u16) {
        self.write(addr, data as u8);
        self.write(addr + 1, (data >> 8) as u8);
    }

    pub fn poll_nmi(&mut self) -> bool {
        self.ppu.poll_nmi()
    }

    fn execute_ppu_read(&mut self, addr: u16) -> u8 {
        let mapped_addr = (addr - 0x2000) % 8;
        match mapped_addr {
            0 => panic!("Attempted to read from write-only PPU register 0x2000"),
            1 => panic!("Attempted to read from write-only PPU register 0x2001"),
            2 => self.ppu.read_ppustatus(),
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
            // 1 => unimplemented!(),
            1 => (),
            2 => panic!("Attempted to write to read-only PPU register 0x2002"),
            // 3 => unimplemented!(),
            3 => (),
            4 => unimplemented!(),
            // 5 => unimplemented!(),
            5 => (),
            6 => self.ppu.write_ppuaddr(data),
            7 => self.ppu.write_ppudata(data),
            _ => unreachable!(),
        }
    }
}
