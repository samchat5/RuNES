use itertools::Itertools;
use std::{cell::RefCell, rc::Rc};

use crate::joypad::Joypad;
use crate::{
    cpu,
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

type CallbackType<'a> = Box<dyn FnMut(&mut PPU, &mut Joypad) + 'a>;

pub struct Bus<'a> {
    cpu_ram: [u8; RAM_SIZE],
    apu_io: [u8; APU_IO_SIZE],
    pub ppu: PPU,
    joypad: Joypad,
    cpu_cycles: u64,
    pub mapper: Rc<RefCell<dyn Mapper>>,
    callback: CallbackType<'a>,
}

impl<'a> Bus<'a> {
    pub fn new<F>(file: File, callback: F) -> Bus<'a>
    where
        F: FnMut(&mut PPU, &mut Joypad) + 'a,
    {
        let mapper = Rc::new(RefCell::new(NROM::new(
            file.prg_rom_area,
            file.chr_rom_area,
        )));
        Bus {
            cpu_ram: [0; RAM_SIZE],
            apu_io: [0; APU_IO_SIZE],
            mapper: mapper.clone(),
            joypad: Joypad::default(),
            ppu: PPU::new(mapper),
            cpu_cycles: 0,
            callback: Box::new(callback),
        }
    }

    pub fn read_trace(&self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.ppu.read_ppudata_trace(addr as usize),
            APU_IO_START..=APU_IO_END => self.apu_io[(addr & 0x001F) as usize],
            _ => self.mapper.borrow().read(addr),
        }
    }
    pub fn read_16_trace(&self, addr: u16) -> u16 {
        let low = self.read_trace(addr);
        let high = self.read_trace(addr + 1);
        (high as u16) << 8 | low as u16
    }

    pub fn get_cycles(&self) -> u64 {
        self.cpu_cycles
    }

    pub fn tick(&mut self, cycles: u64) {
        self.cpu_cycles += cycles;
        let new_frame = self.ppu.tick(cycles * 3);
        if new_frame {
            (self.callback)(&mut self.ppu, &mut self.joypad);
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_read(addr as u16),
            APU_IO_START..=APU_IO_END => self.execute_apu_io_read(addr as u16),
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
            APU_IO_START..=APU_IO_END => self.execute_apu_io_write(addr as u16, data),
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

    fn execute_apu_io_read(&mut self, addr: u16) -> u8 {
        let mapper_addr = (addr - APU_IO_START) % 0x1F;
        match mapper_addr {
            0x16 => self.joypad.read(),
            0..=0x1f => self.apu_io[mapper_addr as usize],
            _ => unreachable!(),
        }
    }

    fn execute_apu_io_write(&mut self, addr: u16, data: u8) {
        let mapper_addr = (addr - APU_IO_START) % 0x1F;
        match mapper_addr {
            0x14 => self.write_oamdma(data),
            0x16 => self.joypad.write(data),
            0..=0x1f => self.apu_io[mapper_addr as usize] = data,
            _ => unreachable!(),
        }
    }

    fn write_oamdma(&mut self, data: u8) {
        let hi = (data as u16) << 8;
        let buffer = (0..256)
            .map(|i| self.read(hi + i as u16))
            .collect_vec()
            .try_into()
            .unwrap();
        self.ppu.write_oamdma(buffer);
    }

    fn execute_ppu_read(&mut self, addr: u16) -> u8 {
        let mapped_addr = (addr - PPU_REG_START) % 8;
        match mapped_addr {
            0 | 1 | 3 | 5 | 6 => {
                println!(
                    "Attempted to read from write-only PPU register 0x200{}",
                    mapped_addr
                );
                0
            }
            2 => self.ppu.read_ppustatus(),
            4 => self.ppu.read_oamdata(),
            7 => self.ppu.read_ppudata(),
            _ => unreachable!(),
        }
    }

    fn execute_ppu_write(&mut self, addr: u16, data: u8) {
        let mapped_addr = (addr - PPU_REG_START) % 8;
        match mapped_addr {
            0 => self.ppu.write_ppuctrl(data),
            1 => self.ppu.write_ppumask(data),
            2 => println!("Attempted to write to read-only PPU register 0x2002"),
            3 => self.ppu.write_oamaddr(data),
            4 => self.ppu.write_oamdata(data),
            5 => self.ppu.write_ppuscroll(data),
            6 => self.ppu.write_ppuaddr(data),
            7 => self.ppu.write_ppudata(data),
            _ => unreachable!(),
        }
    }
}
