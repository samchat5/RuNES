use core::panic;
use std::io::{self, Write};

use bitflags::bitflags;

use crate::config::Config;
use crate::core::ppu::DMAFlag;
use crate::core::{apu::frame_counter::IRQSignal, bus::Bus};

use self::{
    cpu_units::{
        arithmetic::Arithmetic, branches::Branches, flag_changes::FlagChanges,
        inc_dec_ops::IncDecOps, jumps::Jumps, load_store::LoadStore, logical::Logical,
        register_transfer::RegisterTransfer, shift::Shift, stack_ops::StackOps,
        sys_funcs::SysFuncs,
    },
    op::OPS,
    tracer::Loggable,
};

mod cpu_units;
mod op;
mod tracer;

bitflags! {
    pub struct Status: u8 {
        const CARRY = 0x01;
        const ZERO = 0x02;
        const INTERRUPT_DISABLE = 0x04;
        const DECIMAL_MODE = 0x08;
        const BREAK = 0x10;
        const BREAK2 = 0x20;
        const OVERFLOW = 0x40;
        const NEGATIVE = 0x80;
        const STATUS = Self::CARRY.bits()
            | Self::ZERO.bits()
            | Self::INTERRUPT_DISABLE.bits()
            | Self::DECIMAL_MODE.bits()
            | Self::BREAK.bits()
            | Self::OVERFLOW.bits()
            | Self::NEGATIVE.bits();
    }
}

bitflags! {
    pub struct IRQSource: u8 {
        const EXT = 0x01;
        const FRAME_COUNTER = 0x02;
        const DMC = 0x04;
    }
}

pub enum Register {
    X,
    Y,
    A,
    P,
}

#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteXW,
    AbsoluteY,
    AbsoluteYW,
    IndexedIndirect,
    IndirectIndexed,
    IndirectIndexedW,
    Implicit,
    Accumulator,
    Relative,
    Indirect,
}

pub struct CPU<'a> {
    // Registers
    pub x: u8,
    pub y: u8,
    pub acc: u8,
    pub sp: u8,
    pub pc: u16,

    // Status flags
    pub status: Status,

    pub bus: Bus,

    // Logger
    pub sink: Box<dyn Write + Send>,
    logging_enabled: bool,

    // Flags
    irq_flag: IRQSource,
    irq_mask: u8,
    prev_run_irq: bool,
    run_irq: bool,

    start_clock_count: u8,
    end_clock_count: u8,
    master_clock: u64,
    cycle_count: u64,

    need_nmi: bool,
    prev_need_nmi: bool,
    prev_nmi_flag: bool,

    need_halt: bool,
    ppu_offset: u8,
    instr_addr_mode: AddressingMode,
    cpu_write: bool,
    operand: u16,
    sprite_dma_transfer: bool,
    need_dummy_read: bool,
    sprite_dma_offset: u8,

    phantom: std::marker::PhantomData<&'a ()>,
}

impl CPU<'_> {
    pub fn new(bus: Bus) -> Self {
        CPU {
            x: 0,
            y: 0,
            acc: 0,
            sp: 0xFD,
            pc: 0,
            bus,
            status: Status { bits: 0x04 },
            sink: Box::new(io::sink()),
            logging_enabled: false,
            irq_flag: IRQSource { bits: 0 },
            need_halt: false,
            run_irq: false,
            start_clock_count: 6,
            end_clock_count: 6,
            master_clock: 0,
            cycle_count: 0,
            ppu_offset: 0,
            need_nmi: false,
            prev_need_nmi: false,
            prev_run_irq: false,
            instr_addr_mode: AddressingMode::Implicit,
            cpu_write: false,
            operand: 0,
            prev_nmi_flag: false,
            sprite_dma_transfer: false,
            need_dummy_read: false,
            sprite_dma_offset: 0,
            irq_mask: 0,
            phantom: std::marker::PhantomData,
        }
    }

    fn poll_sprite_dma_flag(&mut self) {
        if let DMAFlag::Enabled(x) = self.bus.ppu.sprite_dma_transfer {
            self.sprite_dma_transfer = true;
            self.sprite_dma_offset = x;
            self.need_halt = true
        }
    }

    fn set_sprite_dma_transfer(&mut self, val: bool) {
        self.bus.ppu.sprite_dma_transfer = DMAFlag::Disabled;
        self.sprite_dma_transfer = val;
    }

    fn read_trace(&self, addr: u16) -> u8 {
        self.bus.read_trace(addr)
    }

    fn read_16_trace(&self, addr: u16) -> u16 {
        self.bus.read_16_trace(addr)
    }

    fn read(&mut self, addr: u16) -> u8 {
        let ret = self.bus.read(addr);
        match ret.1 {
            IRQSignal::Set => self.irq_flag.set(IRQSource::FRAME_COUNTER, true),
            IRQSignal::Clear => self.irq_flag.set(IRQSource::FRAME_COUNTER, false),
            IRQSignal::None => {}
        }
        ret.0
    }

    fn write(&mut self, addr: u16, val: u8) {
        match self.bus.write(addr, val, self.cycle_count) {
            IRQSignal::Set => self.irq_flag.set(IRQSource::FRAME_COUNTER, true),
            IRQSignal::Clear => self.irq_flag.set(IRQSource::FRAME_COUNTER, false),
            IRQSignal::None => {}
        }
    }

    fn run_to(&mut self, cyc: u64) {
        self.bus.run_to(cyc);
    }

    pub fn enable_logging(&mut self) {
        self.logging_enabled = true;
    }

    pub fn set_sink(&mut self, stream: Box<dyn Write + Send>) {
        self.sink = stream;
    }

    fn process_pending_dma(&mut self, addr: u16) {
        self.poll_sprite_dma_flag();
        if self.need_halt {
            self.start_cpu_cycle(true);
            self.read(addr);
            self.end_cpu_cycle(true);
            self.need_halt = false;

            let mut sprite_dma_counter = 0u16;
            let mut sprite_read_addr = 0u8;
            let mut read_val = 0u8;
            let skip_dummy_reads = addr == 0x4016 || addr == 0x4017;

            while self.sprite_dma_transfer {
                if self.cycle_count & 0x01 == 0 {
                    self.process_cycle();
                    read_val =
                        self.read(self.sprite_dma_offset as u16 * 0x100 + sprite_read_addr as u16);
                    self.end_cpu_cycle(true);
                    sprite_read_addr = sprite_read_addr.wrapping_add(1);
                    sprite_dma_counter += 1;
                } else if self.sprite_dma_transfer && sprite_dma_counter & 0x01 != 0 {
                    self.process_cycle();
                    self.write(0x2004, read_val);
                    self.end_cpu_cycle(true);
                    sprite_dma_counter += 1;
                    if sprite_dma_counter == 0x200 {
                        self.set_sprite_dma_transfer(false);
                    }
                } else {
                    self.process_cycle();
                    if !skip_dummy_reads {
                        self.read(addr);
                    }
                    self.end_cpu_cycle(true);
                }
            }
        }
    }

    fn process_cycle(&mut self) {
        if self.need_halt {
            self.need_halt = false;
        } else if self.need_dummy_read {
            self.need_dummy_read = false;
        }
        self.start_cpu_cycle(true);
    }

    pub fn memory_read(&mut self, addr: u16) -> u8 {
        self.process_pending_dma(addr);
        self.start_cpu_cycle(true);
        let val = self.read(addr);
        self.end_cpu_cycle(true);
        val
    }

    pub fn memory_read_word(&mut self, addr: u16) -> u16 {
        let lo = self.memory_read(addr);
        let hi = self.memory_read(addr + 1);
        (hi as u16) << 8 | lo as u16
    }

    pub fn memory_write(&mut self, addr: u16, val: u8) {
        self.cpu_write = true;
        self.start_cpu_cycle(false);
        self.write(addr, val);
        self.end_cpu_cycle(false);
        self.cpu_write = false;
    }

    pub fn set_zero_neg_flags(&mut self, val: u8) {
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    pub fn set_register(&mut self, reg: Register, val: u8) {
        match reg {
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::A => self.acc = val,
            _ => panic!("Invalid register"),
        }
        self.set_zero_neg_flags(val);
    }

    fn push(&mut self, data: u8) {
        self.memory_write(0x100 + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.memory_read(0x100 + self.sp as u16)
    }

    fn push_word(&mut self, data: u16) {
        self.push((data >> 8) as u8);
        self.push((data & 0xff) as u8);
    }

    fn pop_word(&mut self) -> u16 {
        let lo = self.pop() as u16;
        let hi = self.pop() as u16;
        (hi << 8) | lo
    }

    pub fn get_absolute_addr_trace(&self, mode: AddressingMode, addr: u16) -> Option<(u16, bool)> {
        match mode {
            AddressingMode::Immediate => Some((addr, false)),
            AddressingMode::ZeroPage => Some((self.read_trace(addr) as u16, false)),
            AddressingMode::ZeroPageX => {
                Some((self.read_trace(addr).wrapping_add(self.x) as u16, false))
            }
            AddressingMode::ZeroPageY => {
                Some((self.read_trace(addr).wrapping_add(self.y) as u16, false))
            }
            AddressingMode::Absolute => Some((self.read_16_trace(addr), false)),
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteXW => {
                let ptr = self.read_16_trace(addr);
                let inc = ptr.wrapping_add(self.x as u16);
                Some((inc, ptr & 0xff00 != inc & 0xff00))
            }
            AddressingMode::AbsoluteY | AddressingMode::AbsoluteYW => {
                let ptr = self.read_16_trace(addr);
                let inc = ptr.wrapping_add(self.y as u16);
                Some((inc, ptr & 0xff00 != inc & 0xff00))
            }
            AddressingMode::IndexedIndirect => {
                let ptr: u8 = self.read_trace(addr).wrapping_add(self.x);
                Some((
                    (self.read_trace(ptr.wrapping_add(1) as u16) as u16) << 8
                        | (self.read_trace(ptr as u16) as u16),
                    false,
                ))
            }
            AddressingMode::IndirectIndexed | AddressingMode::IndirectIndexedW => {
                let ptr = self.read_trace(addr);
                let deref = (self.read_trace((ptr).wrapping_add(1) as u16) as u16) << 8
                    | (self.read_trace(ptr as u16) as u16);
                let inc = deref.wrapping_add(self.y as u16);
                Some((inc, deref & 0xff00 != inc & 0xff00))
            }
            _ => None,
        }
    }

    fn read_byte(&mut self) -> u8 {
        let val = self.memory_read(self.pc);
        self.pc += 1;
        val
    }

    fn read_word(&mut self) -> u16 {
        let val = self.memory_read_word(self.pc);
        self.pc += 2;
        val
    }

    fn get_ind(&mut self) -> u16 {
        let addr = self.operand;
        if (addr & 0xff) == 0xff {
            let lo = self.memory_read(addr);
            let hi = self.memory_read(addr.wrapping_sub(0xff));
            lo as u16 | (hi as u16) << 8
        } else {
            self.memory_read_word(addr)
        }
    }

    fn get_immediate(&mut self) -> u8 {
        self.read_byte()
    }

    fn get_zero_addr(&mut self) -> u8 {
        self.read_byte()
    }

    fn get_zero_x_addr(&mut self) -> u8 {
        let val = self.read_byte();
        self.memory_read(val as u16);
        val.wrapping_add(self.x)
    }

    fn get_zero_y_addr(&mut self) -> u8 {
        let val = self.read_byte();
        self.memory_read(val as u16);
        val.wrapping_add(self.y)
    }

    fn dummy_read(&mut self) {
        self.memory_read(self.pc);
    }

    fn get_ind_addr(&mut self) -> u16 {
        self.read_word()
    }

    fn get_ind_x_addr(&mut self) -> u16 {
        let mut zero = self.read_byte();
        self.memory_read(zero as u16);
        zero = zero.wrapping_add(self.x);
        if zero == 0xff {
            (self.memory_read(0xff) as u16) | ((self.memory_read(0x00) as u16) << 8)
        } else {
            self.memory_read_word(zero as u16)
        }
    }

    fn get_abs_addr(&mut self) -> u16 {
        self.read_word()
    }

    fn check_page_crossed(val_a: u16, val_b: u8) -> bool {
        ((val_a.wrapping_add(val_b as u16)) & 0xFF00) != (val_a & 0xFF00)
    }

    fn check_page_crossed_i8(val_a: u16, val_b: i8) -> bool {
        let sum = match val_b {
            i8::MIN..=0 => val_a.wrapping_sub(val_b.unsigned_abs() as u16),
            1..=i8::MAX => val_a.wrapping_add(val_b as u16),
        };
        (sum & 0xFF00) != (val_a & 0xFF00)
    }

    fn get_ind_y_addr(&mut self, dummy_read: bool) -> u16 {
        let zero = self.read_byte();
        let addr = if zero == 0xFF {
            self.memory_read(0xff) as u16 | ((self.memory_read(0x00) as u16) << 8)
        } else {
            self.memory_read_word(zero as u16)
        };
        let page_crossed = Self::check_page_crossed(addr, self.y);
        if page_crossed || dummy_read {
            let offset = if page_crossed { 0x100 } else { 0 };
            self.memory_read(addr.wrapping_add(self.y as u16).wrapping_sub(offset));
        }
        addr.wrapping_add(self.y as u16)
    }

    fn get_abs_x_addr(&mut self, dummy_read: bool) -> u16 {
        let base_addr = self.read_word();
        let page_crossed = Self::check_page_crossed(base_addr, self.x);
        if page_crossed || dummy_read {
            let offset = if page_crossed { 0x100 } else { 0 };
            self.memory_read(base_addr.wrapping_add(self.x as u16).wrapping_sub(offset));
        }
        base_addr.wrapping_add(self.x as u16)
    }

    fn get_abs_y_addr(&mut self, dummy_read: bool) -> u16 {
        let addr = self.read_word();
        let page_crossed = Self::check_page_crossed(addr, self.y);
        if page_crossed || dummy_read {
            let offset = if page_crossed { 0x100 } else { 0 };
            self.memory_read(addr.wrapping_add(self.y as u16).wrapping_sub(offset));
        }
        addr.wrapping_add(self.y as u16)
    }

    pub fn fetch_operand(&mut self) -> u16 {
        match self.instr_addr_mode {
            AddressingMode::Accumulator | AddressingMode::Implicit => {
                self.dummy_read();
                0
            }
            AddressingMode::Immediate | AddressingMode::Relative => self.get_immediate() as u16,
            AddressingMode::ZeroPage => self.get_zero_addr() as u16,
            AddressingMode::ZeroPageX => self.get_zero_x_addr() as u16,
            AddressingMode::ZeroPageY => self.get_zero_y_addr() as u16,
            AddressingMode::Indirect => self.get_ind_addr(),
            AddressingMode::IndexedIndirect => self.get_ind_x_addr(),
            AddressingMode::IndirectIndexed => self.get_ind_y_addr(false),
            AddressingMode::IndirectIndexedW => self.get_ind_y_addr(true),
            AddressingMode::Absolute => self.get_abs_addr(),
            AddressingMode::AbsoluteX => self.get_abs_x_addr(false),
            AddressingMode::AbsoluteXW => self.get_abs_x_addr(true),
            AddressingMode::AbsoluteY => self.get_abs_y_addr(false),
            AddressingMode::AbsoluteYW => self.get_abs_y_addr(true),
        }
    }

    pub fn reset(&mut self) {
        self.bus.set_nmi_generated(false);
        self.irq_flag = IRQSource { bits: 0 };
        self.need_halt = false;
        self.irq_mask = 0xff;

        self.pc = self.read(0xFFFC) as u16 | ((self.read(0xFFFD) as u16) << 8);

        self.acc = 0;
        self.x = 0;
        self.sp = 0xFD;
        self.y = 0;
        self.status = Status { bits: 0x04 };

        self.run_irq = false;

        self.cycle_count = 0u64.wrapping_sub(1);
        self.master_clock = 0;
        self.ppu_offset = 1;

        self.master_clock += 12;

        (0..8).for_each(|_| {
            self.start_cpu_cycle(true);
            self.end_cpu_cycle(true);
        });
    }

    fn get_nmi_flag(&self) -> bool {
        self.bus.ppu.nmi_generated
    }

    fn start_cpu_cycle(&mut self, is_read: bool) {
        self.master_clock += if is_read {
            self.start_clock_count.wrapping_sub(1)
        } else {
            self.start_clock_count.wrapping_add(1)
        } as u64;
        self.cycle_count = self.cycle_count.wrapping_add(1);
        self.run_to(self.master_clock - self.ppu_offset as u64);
        if self.bus.apu.clock() {
            self.irq_flag.set(IRQSource::FRAME_COUNTER, true);
        }
    }

    fn end_cpu_cycle(&mut self, is_read: bool) {
        self.master_clock += if is_read {
            self.end_clock_count.wrapping_add(1)
        } else {
            self.end_clock_count.wrapping_sub(1)
        } as u64;
        self.run_to(self.master_clock - self.ppu_offset as u64);

        self.prev_need_nmi = self.need_nmi;

        if !self.prev_nmi_flag && self.get_nmi_flag() {
            self.need_nmi = true;
        }
        self.prev_nmi_flag = self.get_nmi_flag();

        self.prev_run_irq = self.run_irq;
        self.run_irq = (self.irq_flag.bits & self.irq_mask) > 0
            && !self.status.contains(Status::INTERRUPT_DISABLE)
    }

    fn get_op_code(&mut self) -> u8 {
        let op_code = self.memory_read(self.pc);
        self.pc += 1;
        op_code
    }

    fn get_operand_val(&mut self) -> u8 {
        match self.instr_addr_mode {
            AddressingMode::Accumulator
            | AddressingMode::Implicit
            | AddressingMode::Immediate
            | AddressingMode::Relative => self.operand as u8,
            _ => self.memory_read(self.operand),
        }
    }

    pub fn get_frame_hash(&self) -> u64 {
        self.bus.ppu.curr_frame.get_hash()
    }

    pub fn run_for_cycles(&mut self, cycles: u64) {
        while self.cycle_count < cycles {
            self.run();
        }
    }

    pub fn run_until_frame(&mut self) {
        let frame_num = self.bus.ppu.frame_count;
        while self.bus.ppu.frame_count == frame_num
            && self.cycle_count < Config::get_int("max_cycles", i64::MAX) as u64
        {
            self.run();
        }
    }

    fn run(&mut self) {
        self.log();
        let opcode = self.get_op_code();

        let searched_op = OPS.binary_search_by_key(&opcode, |op| op.hex);
        let op = if searched_op.is_err() {
            println!("Invalid opcode: {:02X}", opcode);
            &OPS[OPS.binary_search_by_key(&"NOP", |op| op.name).unwrap()]
        } else {
            &OPS[OPS.binary_search_by_key(&opcode, |op| op.hex).unwrap()]
        };
        self.instr_addr_mode = op.addressing_mode;
        self.operand = self.fetch_operand();

        match op.name {
            "AND" => self.and(),
            "ADC" => self.adc(),
            "*ANC" => self.anc(),
            "*ARR" => self.arr(),
            "ASL" => self.asl(),
            "*ASR" => self.asr(),
            "*AXS" => self.axs(),
            "BCC" => self.bcc(),
            "BCS" => self.bcs(),
            "BEQ" => self.beq(),
            "BIT" => self.bit(),
            "BPL" => self.bpl(),
            "BMI" => self.bmi(),
            "BNE" => self.bne(),
            "BRK" => self.brk(),
            "BVC" => self.bvc(),
            "BVS" => self.bvs(),
            "CLC" => self.clc(),
            "CLD" => self.cld(),
            "CLI" => self.cli(),
            "CLV" => self.clv(),
            "CMP" => self.cmp(),
            "CPX" => self.cpx(),
            "CPY" => self.cpy(),
            "*DCP" => self.dcp(),
            "DEC" => self.dec(),
            "DEX" => self.dex(),
            "DEY" => self.dey(),
            "EOR" => self.eor(),
            "INC" => self.inc(),
            "INX" => self.inx(),
            "INY" => self.iny(),
            "*ISB" => self.isb(),
            "JMP" => self.jmp(),
            "JSR" => self.jsr(),
            "*LAX" => self.lax(),
            "LDA" => self.lda(),
            "LDX" => self.ldx(),
            "LDY" => self.ldy(),
            "LSR" => self.lsr(),
            "*LXA" => self.lxa(),
            "NOP" | "*NOP" => self.nop(),
            "ORA" => self.ora(),
            "PHA" => self.pha(),
            "PHP" => self.php(),
            "PLA" => self.pla(),
            "PLP" => self.plp(),
            "ROL" => self.rol(),
            "ROR" => self.ror(),
            "*RLA" => self.rla(),
            "*RRA" => self.rra(),
            "RTI" => self.rti(),
            "RTS" => self.rts(),
            "*SAX" => self.sax(),
            "SBC" | "*SBC" => self.sbc(),
            "SEC" => self.sec(),
            "SED" => self.sed(),
            "SEI" => self.sei(),
            "*SHX" => self.shx(),
            "*SHY" => self.shy(),
            "*SLO" => self.slo(),
            "*SRE" => self.sre(),
            "STA" => self.sta(),
            "STY" => self.sty(),
            "STX" => self.stx(),
            "TAX" => self.tax(),
            "TAY" => self.tay(),
            "TSX" => self.tsx(),
            "TXA" => self.txa(),
            "TXS" => self.txs(),
            "TYA" => self.tya(),
            _ => panic!("Unknown opcode: {:02x}", opcode),
        }

        if self.prev_run_irq || self.prev_need_nmi {
            self.irq();
        }
    }
}
