use std::{cell::RefCell, rc::Rc};

use crate::core::apu::base_channel::AudioChannel;
use crate::core::apu::frame_counter::IRQSignal;
use crate::core::apu::APU;
use crate::core::joypad::Joypad;
use crate::core::mappers::{MapperFactory, SharedMapper};
use crate::{core::ppu::PPU, ines_parser::File};

const RAM_SIZE: usize = 0x0800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;
const APU_IO_START: u16 = 0x4000;
const APU_IO_END: u16 = 0x401F;

pub struct Bus {
    cpu_ram: [u8; RAM_SIZE],
    pub ppu: PPU,
    pub apu: APU,
    pub joypad: Joypad,
    mapper: SharedMapper,
}

impl Bus {
    pub fn new(file: &File) -> Bus {
        let mapper = Rc::new(RefCell::new(MapperFactory::from_file(file)));
        println!("here bus");
        Bus {
            cpu_ram: [0; RAM_SIZE],
            mapper: mapper.clone(),
            joypad: Joypad::default(),
            ppu: PPU::new(mapper),
            apu: APU::new(),
        }
    }

    pub fn read_trace(&self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.ppu.read_ppudata_trace(addr as usize),
            APU_IO_START..=APU_IO_END => self.read_apu_trace(addr),
            _ => self.mapper.borrow().read(addr),
        }
    }

    pub fn read_apu_trace(&self, addr: u16) -> u8 {
        let mapped_addr = (addr - APU_IO_START) % 0x1F;
        match mapped_addr {
            0x16 => self.joypad.read_trace(),
            // Controller 2 data disabled -- always return 0
            0x17 => 0,
            0x15 => self.apu.read_status_trace(),
            _ => self.ppu.open_bus,
        }
    }

    pub fn read_16_trace(&self, addr: u16) -> u16 {
        let low = self.read_trace(addr);
        let high = self.read_trace(addr + 1);
        (high as u16) << 8 | low as u16
    }

    pub fn read(&mut self, addr: u16) -> (u8, IRQSignal) {
        let mut signal = IRQSignal::None;
        let val = match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize],
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_read(addr),
            APU_IO_START..=APU_IO_END => {
                let ret = self.execute_apu_io_read(addr);
                signal = ret.1;
                ret.0
            }
            _ => self.mapper.borrow().read(addr),
        };
        (val, signal)
    }

    pub fn write(&mut self, addr: u16, data: u8, cpu_cycle: u64) -> IRQSignal {
        let mut signal = IRQSignal::None;
        match addr {
            RAM_START..=RAM_END => self.cpu_ram[(addr & 0x07FF) as usize] = data,
            PPU_REG_START..=PPU_REG_END => self.execute_ppu_write(addr, data),
            APU_IO_START..=APU_IO_END => signal = self.execute_apu_io_write(addr, data, cpu_cycle),
            _ => self.mapper.to_owned().borrow_mut().write(addr, data),
        }
        signal
    }

    fn execute_apu_io_read(&mut self, addr: u16) -> (u8, IRQSignal) {
        let mapper_addr = (addr - APU_IO_START) % 0x1F;
        let mut signal = IRQSignal::None;
        let val = match mapper_addr {
            0x16 => self.joypad.read(),
            // Controller 2 data disabled -- always return 0
            0x17 => 0,
            0x15 => {
                let ret = self.apu.read_status();
                signal = ret.1;
                ret.0
            }
            _ => self.ppu.open_bus,
        };
        (val, signal)
    }

    fn execute_apu_io_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) -> IRQSignal {
        let mapped_addr = (addr - APU_IO_START) % 0x1F;
        let mut signal = IRQSignal::None;
        match mapped_addr {
            0x00 => self.apu.write_ctrl(&AudioChannel::Pulse1, data),
            0x01 => self.apu.write_sweep(&AudioChannel::Pulse1, data),
            0x02 => self.apu.write_timer_lo(&AudioChannel::Pulse1, data),
            0x03 => self.apu.write_timer_hi(&AudioChannel::Pulse1, data),
            0x04 => self.apu.write_ctrl(&AudioChannel::Pulse2, data),
            0x05 => self.apu.write_sweep(&AudioChannel::Pulse2, data),
            0x06 => self.apu.write_timer_lo(&AudioChannel::Pulse2, data),
            0x07 => self.apu.write_timer_hi(&AudioChannel::Pulse2, data),
            0x08..=0x13 => {}
            0x14 => self.ppu.write_oamdma(data),
            0x15 => self.apu.write_status(data),
            0x16 => self.joypad.write(data),
            0x17 => signal = self.apu.write_frame_counter(data),
            _ => unreachable!(),
        }
        signal
    }

    fn execute_ppu_read(&mut self, addr: u16) -> u8 {
        let mapped_addr = (addr - PPU_REG_START) % 8;
        let mut open_bus_mask = 0xff;
        let ret = match mapped_addr {
            0 | 1 | 3 | 5 | 6 => {
                println!(
                    "Attempted to read from write-only PPU register 0x200{}",
                    mapped_addr
                );
                0
            }
            2 => self.ppu.read_ppustatus(&mut open_bus_mask),
            4 => {
                open_bus_mask = 0x0;
                self.ppu.read_oamdata()
            }
            7 => self.ppu.read_ppudata(&mut open_bus_mask),
            _ => unreachable!(),
        };
        self.ppu.apply_open_bus(open_bus_mask, ret)
    }

    fn execute_ppu_write(&mut self, addr: u16, data: u8) {
        if addr != 0x4014 {
            self.ppu.set_open_bus(0xff, data);
        }
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

    pub fn set_nmi_generated(&mut self, nmi: bool) {
        self.ppu.nmi_generated = nmi;
    }

    pub(crate) fn run_to(&mut self, cyc: u64) {
        self.ppu.run_to(cyc);
    }
}
