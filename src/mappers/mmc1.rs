use crate::mappers::{Mapper, Mirroring};

enum PRGMode {
    PRG16k,
    PRG32k,
}

enum CHRMode {
    CHR8k,
    CHR4k,
}

enum Register {
    Control,
    CHRBank0,
    CHRBank1,
    PRGBank,
}

enum SlotSelect {
    Slot0,
    Slot1,
}

#[derive(Default)]
struct State {
    control_reg: u8,
    chr_bank_0_reg: u8,
    chr_bank_1_reg: u8,
    prg_bank_reg: u8,
}

#[derive(Default)]
pub struct MMC1 {
    // write_buffer: u8,
    shift_count: u8,
    // chr_mode: CHRMode,
    // prg_mode: PRGMode,

    // chr_reg_0: u8,
    // chr_reg_1: u8,
    // prg_reg: u8,

    // last_write_cycle: u64,
    state: State,
    // last_chr_reg: Register,
    temp_reg: u8,

    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl MMC1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            prg_rom,
            chr_rom,
            ..Self::default()
        }
    }

    fn write_control(&mut self, data: u8) {
        let reset = (data >> 7) & 1;
        if reset == 1 {
            self.temp_reg = 0;
            self.shift_count = 0;
            self.state.control_reg |= 0b0000_1100;
        } else {
            let bit = data & 1;
            self.temp_reg |= bit << self.shift_count;

            if self.shift_count == 4 {
                self.state.control_reg = self.temp_reg;
                self.temp_reg = 0;
                self.shift_count = 0;
            } else {
                self.shift_count += 1;
            }
        }
    }
}

impl Mapper for MMC1 {
    fn get_mirroring(&self) -> Mirroring {
        match self.state.control_reg & 0b11 {
            0 => Mirroring::Horizontal,
            1 => Mirroring::Vertical,
            2 => Mirroring::FourScreen,
            3 => Mirroring::FourScreen,
            _ => unreachable!(),
        }
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        match self.state.control_reg & 0b1000 {
            0 => self.chr_rom[addr as usize],
            1 => todo!(),
            _ => unreachable!(),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {
                self.write_control(data);
            }
            _ => todo!(),
        }
        todo!("Implement consecutive write logic for MMC1");
    }

    fn write_chr_rom(&mut self, addr: u16, data: u8) {
        todo!()
    }
}
