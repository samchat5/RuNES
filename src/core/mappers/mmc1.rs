use crate::core::mappers::{Mapper, Mirroring};

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

struct State {
    control_reg: u8,
    chr_bank_0_reg: u8,
    chr_bank_1_reg: u8,
    prg_bank_reg: u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            control_reg: 0b0000_1100,
            chr_bank_0_reg: 0,
            chr_bank_1_reg: 0,
            prg_bank_reg: 0,
        }
    }
}

pub struct MMC1 {
    // CPU BANKS -----------------------------------------------------------------------------------
    // $6000-7FFF: 8 KB PRG RAM bank (optional)
    // $8000-BFFF: 16 KB PRG ROM bank, either switchable or fixed to the first bank
    // $C000-FFFF: 16 KB PRG ROM bank, either fixed to the last bank or switchable

    // PPU BANKS -----------------------------------------------------------------------------------
    // $0000-0FFF: 4 KB switchable CHR bank
    // $1000-1FFF: 4 KB switchable CHR bank

    // REGISTERS -----------------------------------------------------------------------------------
    // $8000-9FFF:  [...C PSMM]
    //   C = CHR Mode (0=8k mode, 1=4k mode)
    //   P = PRG Size (0=32k mode, 1=16k mode)
    //   S = Slot select:
    //       0 = $C000 swappable, $8000 fixed to page $00 (mode A)
    //       1 = $8000 swappable, $C000 fixed to page $0F (mode B)
    //       This bit is ignored when 'P' is clear (32k mode)
    //   M = Mirroring control:
    //       %00 = 1ScA
    //       %01 = 1ScB
    //       %10 = Vert
    //       %11 = Horz
    //
    // $A000-BFFF:  [...C CCCC]
    //   CHR Reg 0
    //
    // $C000-DFFF:  [...C CCCC]
    //   CHR Reg 1
    //
    // $E000-FFFF:  [...W PPPP]
    //   W = WRAM Disable (0=enabled, 1=disabled)
    //   P = PRG Reg

    // When writing to $8000-FFFF, it gets written to this register. When writen to 5 times, the
    // value is copied over to the control register
    temp_reg: u8,
    shift_count: u8,
    state: State,
    prg_ram: Vec<u8>,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    has_chr_ram: bool,
    nametables: [[u8; 0x400]; 2],
}

impl MMC1 {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Option<Vec<u8>>,
        prg_ram_size: usize,
        eeprom_size: usize,
        has_battery: bool,
        _mirroring: u8,
    ) -> Self {
        let mut prg_ram_size = prg_ram_size;
        if prg_ram_size == 0 && eeprom_size > 0 {
            prg_ram_size = eeprom_size;
        } else if has_battery {
            prg_ram_size = 0x2000;
        }
        let has_chr_ram = chr_rom.is_none();

        Self {
            prg_ram: vec![0; prg_ram_size],
            prg_rom,
            chr_rom: chr_rom.unwrap_or_else(|| vec![0; 0x2000]),
            temp_reg: 0,
            has_chr_ram,
            shift_count: 0,
            state: State::default(),
            nametables: [[0; 0x400]; 2],
        }
    }

    fn get_chr_mode(&self) -> CHRMode {
        match (self.state.control_reg >> 4) & 1 {
            0 => CHRMode::CHR8k,
            1 => CHRMode::CHR4k,
            _ => unreachable!(),
        }
    }

    fn get_prg_mode(&self) -> PRGMode {
        match (self.state.control_reg >> 3) & 1 {
            0 => PRGMode::PRG32k,
            1 => PRGMode::PRG16k,
            _ => unreachable!(),
        }
    }

    fn get_slot_select(&self) -> SlotSelect {
        match (self.state.control_reg >> 2) & 1 {
            0 => SlotSelect::Slot0,
            1 => SlotSelect::Slot1,
            _ => unreachable!(),
        }
    }

    fn get_wram_disable(&self) -> bool {
        self.state.prg_bank_reg >> 4 == 1
    }

    fn get_prg_bank(&self) -> usize {
        (self.state.prg_bank_reg & 0b1111) as usize
    }

    fn get_mut_ref_reg(&mut self, register: Register) -> &mut u8 {
        match register {
            Register::Control => &mut self.state.control_reg,
            Register::CHRBank0 => &mut self.state.chr_bank_0_reg,
            Register::CHRBank1 => &mut self.state.chr_bank_1_reg,
            Register::PRGBank => &mut self.state.prg_bank_reg,
        }
    }

    fn write_reg(&mut self, data: u8, register: Register) {
        if (data >> 7) & 1 == 1 {
            self.temp_reg = 0;
            self.shift_count = 0;
            self.state.control_reg |= 0b0000_1100;
        } else {
            let data_bit = data & 1;
            self.temp_reg |= data_bit << self.shift_count;
            self.shift_count += 1;
            if self.shift_count == 5 {
                *(self.get_mut_ref_reg(register)) = self.temp_reg;
                self.temp_reg = 0;
                self.shift_count = 0;
            }
        }
    }

    fn get_page_cnt(&self) -> usize {
        match self.get_prg_mode() {
            PRGMode::PRG32k => self.prg_rom.len() / 0x8000,
            PRGMode::PRG16k => self.prg_rom.len() / 0x4000,
        }
    }
}

impl Mapper for MMC1 {
    fn get_mirroring(&self) -> Mirroring {
        match self.state.control_reg & 0b11 {
            0 => Mirroring::SingleScreenA,
            1 => Mirroring::SingleScreenB,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        }
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        match self.get_chr_mode() {
            CHRMode::CHR8k => {
                let page = (self.state.chr_bank_0_reg >> 1) as usize;
                let idx = (page * 8192) + addr as usize;
                self.chr_rom[idx % self.chr_rom.len()]
            }
            CHRMode::CHR4k => match addr {
                0x0000..=0x0FFF => {
                    let page = self.state.chr_bank_0_reg as usize;
                    let idx = (page * 4096) + addr as usize;
                    self.chr_rom[idx % self.chr_rom.len()]
                }
                0x1000..=0x1FFF => {
                    let page = self.state.chr_bank_1_reg as usize;
                    let idx = (page * 4096) + (addr - 0x1000) as usize;
                    self.chr_rom[idx % self.chr_rom.len()]
                }
                _ => panic!("Invalid CHR read addr {:#X}", addr),
            },
        }
    }

    fn read(&self, addr: u16) -> u8 {
        if self.get_wram_disable() && (0x6000..=0x7fff).contains(&addr) {
            println!("WRAM disabled, cannot read from PRG");
            0
        } else {
            self.read_trace(addr)
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if self.get_wram_disable() && (0x6000..=0x7fff).contains(&addr) {
            println!("WRAM disabled, cannot write to PRG");
        } else {
            match addr {
                0x6000..=0x7FFF => {
                    if !self.prg_ram.is_empty() {
                        let idx = ((addr - 0x6000) as usize) % self.prg_ram.len();
                        self.prg_ram[idx] = data;
                    } else {
                        println!("Attempted to write to non-existent PRG RAM");
                    }
                }
                0x8000..=0x9FFF => self.write_reg(data, Register::Control),
                0xA000..=0xBFFF => self.write_reg(data, Register::CHRBank0),
                0xC000..=0xDFFF => self.write_reg(data, Register::CHRBank1),
                0xE000..=0xFFFF => self.write_reg(data, Register::PRGBank),
                _ => println!("Invalid write address: {:#X}", addr),
            }
        }
        // todo!("Implement consecutive write logic for MMC1");
    }

    fn write_chr_rom(&mut self, addr: u16, data: u8) {
        if self.has_chr_ram {
            match self.get_chr_mode() {
                CHRMode::CHR8k => {
                    let page = (self.state.chr_bank_0_reg >> 1) as usize;
                    let idx = page * 8192 + addr as usize;
                    let len = self.chr_rom.len();
                    self.chr_rom[idx % len] = data;
                }
                CHRMode::CHR4k => match addr {
                    0x0000..=0x0FFF => {
                        let page = self.state.chr_bank_0_reg as usize;
                        self.chr_rom[(page * 4096) + addr as usize] = data;
                    }
                    0x1000..=0x1FFF => {
                        let page = self.state.chr_bank_1_reg as usize;
                        self.chr_rom[(page * 4096) + (addr - 0x1000) as usize] = data;
                    }
                    _ => panic!("Invalid CHR read addr {:#X}", addr),
                },
            }
        }
    }

    fn write_nametable_idx(&mut self, idx: usize, addr: u16, val: u8) {
        self.nametables[idx][addr as usize] = val;
    }

    fn read_nametable_idx(&self, idx: usize, addr: u16) -> u8 {
        self.nametables[idx][addr as usize]
    }

    fn read_trace(&self, addr: u16) -> u8 {
        if (0x6000..=0x7FFF).contains(&addr) {
            if !self.prg_ram.is_empty() {
                let idx = (addr - 0x6000) as usize;
                self.prg_ram[idx % self.prg_ram.len()]
            } else {
                println!("Attempted to read from PRG RAM, but it is not mapped");
                0
            }
        } else {
            match self.get_prg_mode() {
                PRGMode::PRG32k => {
                    let mut page = self.get_prg_bank() >> 1;
                    if page >= self.get_page_cnt() {
                        page &= self.get_page_cnt() - 1;
                    }
                    self.prg_rom[(page * 32768) + (addr - 0x8000) as usize]
                }
                _ => {
                    let (mut page, offset): (usize, usize) = match (self.get_slot_select(), addr) {
                        (SlotSelect::Slot0, 0x8000..=0xBFFF) => (0, 0x8000),
                        (SlotSelect::Slot0, 0xC000..=0xFFFF) => (self.get_prg_bank(), 0xC000),
                        (_, 0x8000..=0xBFFF) => (self.get_prg_bank(), 0x8000),
                        (_, 0xC000..=0xFFFF) => (0x0F & (self.get_page_cnt() - 1), 0xC000),
                        _ => {
                            println!("Invalid read address: {:#X}", addr);
                            return 0;
                        }
                    };
                    if page >= self.get_page_cnt() {
                        page &= self.get_page_cnt() - 1;
                    }
                    self.prg_rom[(page * 16384) + addr as usize - offset]
                }
            }
        }
    }
}
