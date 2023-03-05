use core::panic;
use std::io::{self, Write};

use bitflags::bitflags;

use crate::bus::Bus;

use self::{
    cpu_units::{
        arithmetic::Arithmetic, branches::Branches, flag_changes::FlagChanges,
        inc_dec_ops::IncDecOps, jumps::Jumps, load_store::LoadStore, logical::Logical,
        register_transfer::RegisterTransfer, shift::Shift, stack_ops::StackOps,
        sys_funcs::SysFuncs,
    },
    op::OPS,
    // tracer::Loggable,
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
        const STATUS = Self::CARRY.bits
            | Self::ZERO.bits
            | Self::INTERRUPT_DISABLE.bits
            | Self::DECIMAL_MODE.bits
            | Self::BREAK.bits
            | Self::OVERFLOW.bits
            | Self::NEGATIVE.bits;
    }
}

const STACK_BASE: u16 = 0x0100;

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
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
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

    // Memory
    pub bus: Bus<'a>,

    // Logger
    pub sink: Box<dyn Write + Send>,
}

impl<'a> CPU<'a> {
    pub fn new(bus: Bus<'a>) -> CPU<'a> {
        CPU {
            x: 0,
            y: 0,
            acc: 0,
            sp: 0xFD,
            pc: 0,
            status: Status { bits: 0x24 },
            bus,
            sink: Box::new(io::sink()),
        }
    }

    pub fn set_sink(&mut self, stream: Box<dyn Write + Send>) {
        self.sink = stream;
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.bus.write(addr, val);
    }

    pub fn read_16(&mut self, addr: u16) -> u16 {
        self.bus.read_16(addr)
    }

    fn stack_push(&mut self, data: u8) {
        self.write(STACK_BASE + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(STACK_BASE + self.sp as u16)
    }

    fn stack_push_16(&mut self, data: u16) {
        self.stack_push((data >> 8) as u8);
        self.stack_push((data & 0xff) as u8);
    }

    fn stack_pop_16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        (hi << 8) | lo
    }

    pub fn get_absolute_addr(&mut self, mode: AddressingMode, addr: u16) -> Option<(u16, bool)> {
        match mode {
            AddressingMode::Immediate => Some((addr, false)),
            AddressingMode::ZeroPage => Some((self.read(addr) as u16, false)),
            AddressingMode::ZeroPageX => Some((self.read(addr).wrapping_add(self.x) as u16, false)),
            AddressingMode::ZeroPageY => Some((self.read(addr).wrapping_add(self.y) as u16, false)),
            AddressingMode::Absolute => Some((self.read_16(addr) as u16, false)),
            AddressingMode::AbsoluteX => {
                let ptr = self.read_16(addr);
                let inc = ptr.wrapping_add(self.x as u16);
                Some((inc, ptr & 0xff00 != inc & 0xff00))
            }
            AddressingMode::AbsoluteY => {
                let ptr = self.read_16(addr);
                let inc = ptr.wrapping_add(self.y as u16);
                Some((inc, ptr & 0xff00 != inc & 0xff00))
            }
            AddressingMode::IndexedIndirect => {
                let ptr: u8 = self.read(addr).wrapping_add(self.x);
                Some((
                    (self.read(ptr.wrapping_add(1) as u16) as u16) << 8
                        | (self.read(ptr as u16) as u16),
                    false,
                ))
            }
            AddressingMode::IndirectIndexed => {
                let ptr = self.read(addr);
                let deref = (self.read((ptr as u8).wrapping_add(1) as u16) as u16) << 8
                    | (self.read(ptr as u16) as u16);
                let inc = deref.wrapping_add(self.y as u16);
                Some((inc, deref & 0xff00 != inc & 0xff00))
            }
            _ => None,
        }
    }

    pub fn get_operand_addr(&mut self, mode: AddressingMode) -> Option<u16> {
        self.get_absolute_addr(mode, self.pc).map(|(addr, _)| addr)
    }

    pub fn reset(&mut self) {
        let val = self.read_16(0xFFFC);
        self.reset_with_val(val);
    }

    pub fn reset_with_val(&mut self, val: u16) {
        self.pc = val;
        self.bus.tick(7);
    }

    pub fn run(&mut self, cycles: u64) {
        // self.log();
        while self.bus.get_cycles() < cycles {
            if self.bus.poll_nmi() {
                self.nmi();
            }

            let opcode = self.read(self.pc);
            let op = &OPS[OPS.binary_search_by_key(&opcode, |op| op.hex).unwrap()];

            self.pc += 1;

            let pc_copy = self.pc;

            match op.name {
                "AND" => self.and(op.addressing_mode),
                "ADC" => self.adc(op.addressing_mode),
                "ASL" => self.asl(op.addressing_mode),
                "BCC" => self.bcc(),
                "BCS" => self.bcs(),
                "BEQ" => self.beq(),
                "BIT" => self.bit(op.addressing_mode),
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
                "CMP" => self.cmp(op.addressing_mode),
                "CPX" => self.cpx(op.addressing_mode),
                "CPY" => self.cpy(op.addressing_mode),
                "*DCP" => self.dcp(op.addressing_mode),
                "DEC" => self.dec(op.addressing_mode),
                "DEX" => self.dex(),
                "DEY" => self.dey(),
                "EOR" => self.eor(op.addressing_mode),
                "INC" => self.inc(op.addressing_mode),
                "INX" => self.inx(),
                "INY" => self.iny(),
                "*ISB" => self.isb(op.addressing_mode),
                "JMP" => self.jmp(op.addressing_mode),
                "JSR" => self.jsr(),
                "*LAX" => self.lax(op.addressing_mode),
                "LDA" => self.lda(op.addressing_mode),
                "LDX" => self.ldx(op.addressing_mode),
                "LDY" => self.ldy(op.addressing_mode),
                "LSR" => self.lsr(op.addressing_mode),
                "NOP" | "*NOP" => self.nop(op.addressing_mode),
                "ORA" => self.ora(op.addressing_mode),
                "PHA" => self.pha(),
                "PHP" => self.php(),
                "PLA" => self.pla(),
                "PLP" => self.plp(),
                "ROL" => self.rol(op.addressing_mode),
                "ROR" => self.ror(op.addressing_mode),
                "*RLA" => self.rla(op.addressing_mode),
                "*RRA" => self.rra(op.addressing_mode),
                "RTI" => self.rti(),
                "RTS" => self.rts(),
                "*SAX" => self.sax(op.addressing_mode),
                "SBC" | "*SBC" => self.sbc(op.addressing_mode),
                "SEC" => self.sec(),
                "SED" => self.sed(),
                "SEI" => self.sei(),
                "*SLO" => self.slo(op.addressing_mode),
                "*SRE" => self.sre(op.addressing_mode),
                "STA" => self.sta(op.addressing_mode),
                "STY" => self.sty(op.addressing_mode),
                "STX" => self.stx(op.addressing_mode),
                "TAX" => self.tax(),
                "TAY" => self.tay(),
                "TSX" => self.tsx(),
                "TXA" => self.txa(),
                "TXS" => self.txs(),
                "TYA" => self.tya(),
                _ => panic!("Unknown opcode: {:02x}", opcode),
            }

            // In case of jumps and branches
            if pc_copy == self.pc {
                self.pc += op.size - 1;
            }

            self.bus.tick(op.cycles as u64);

            // self.log();
        }
    }
}
