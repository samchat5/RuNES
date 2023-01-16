use core::panic;
use std::io::prelude::*;

use crate::ines_parser::File;
use bitflags::bitflags;

use super::{
    op::OPS,
    tracer::trace,
    traits::{
        ArithOp, Arithmetic, Branches, FlagChanges, IncDec, IncDecOps, Jumps, LoadStore, Logical,
        LogicalOp, RegisterTransfer, Shift, ShiftOp, StackOps, SysFuncs,
    },
};

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

pub struct CPU {
    // Registers
    pub x: u8,
    pub y: u8,
    pub acc: u8,
    pub sp: u8,
    pub pc: u16,

    // Status flags
    pub status: Status,

    // Memory
    memory: [u8; 65536],

    // Cycles
    pub cycles: u64,
}

trait ResetAddr {
    fn reset(&mut self);
}

impl LoadStore for CPU {
    fn ld(&mut self, mode: AddressingMode, regs: Vec<Register>) {
        let (val, inc_cycles) = self.get_absolute_addr(mode, self.pc).unwrap();
        if inc_cycles {
            self.cycles += 1;
        }
        let val = self.read(val);
        regs.iter().for_each(|reg| match reg {
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::A => self.acc = val,
            _ => panic!("Invalid register for load"),
        });
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    fn st(&mut self, mode: AddressingMode, regs: Vec<Register>) {
        self.write(
            self.get_operand_addr(mode).unwrap(),
            regs.iter().fold(0xff, |acc, r| {
                acc & match r {
                    Register::X => self.x,
                    Register::Y => self.y,
                    Register::A => self.acc,
                    _ => panic!("Invalid register for store"),
                }
            }),
        );
    }
}

impl RegisterTransfer for CPU {
    fn transfer(&mut self, from: Register, to: Register) {
        match (from, to) {
            (Register::A, Register::X) => {
                self.x = self.acc;
            }
            (Register::A, Register::Y) => {
                self.y = self.acc;
            }
            (Register::X, Register::A) => {
                self.acc = self.x;
            }
            (Register::Y, Register::A) => {
                self.acc = self.y;
            }
            _ => panic!("Invalid transfer"),
        }
        self.status.set(Status::ZERO, self.acc == 0);
        self.status.set(Status::NEGATIVE, self.acc >> 7 == 1);
    }
}

impl StackOps for CPU {
    fn tsx(&mut self) {
        self.x = self.sp;
        self.status.set(Status::ZERO, self.x == 0);
        self.status.set(Status::NEGATIVE, self.x & 0x80 != 0);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn ph(&mut self, reg: Register) {
        self.stack_push(match reg {
            Register::A => self.acc,
            Register::P => (self.status | Status::BREAK | Status::BREAK2).bits,
            _ => panic!("Invalid register for push"),
        });
    }

    fn pl(&mut self, reg: Register) {
        match reg {
            Register::A => {
                self.acc = self.stack_pop();
                self.status.set(Status::ZERO, self.acc == 0);
                self.status.set(Status::NEGATIVE, self.acc & 0x80 != 0);
            }
            Register::P => {
                self.status.bits = self.stack_pop() & !Status::BREAK.bits | Status::BREAK2.bits
            }
            _ => panic!("Invalid register for pull"),
        }
    }
}

impl Logical for CPU {
    fn bit_op(&mut self, mode: AddressingMode, op: LogicalOp, does_inc_cycles: bool) {
        let (addr, inc_cycles) = self.get_absolute_addr(mode, self.pc).unwrap();
        let val = self.read(addr);
        if inc_cycles && does_inc_cycles {
            self.cycles += 1;
        }
        self.acc = match op {
            LogicalOp::AND => self.acc & val,
            LogicalOp::ORA => self.acc | val,
            LogicalOp::EOR => self.acc ^ val,
        };
        self.status.set(Status::ZERO, self.acc == 0);
        self.status.set(Status::NEGATIVE, self.acc & 0x80 != 0);
    }

    fn bit(&mut self, mode: AddressingMode) {
        let val = self.read(self.get_operand_addr(mode).unwrap());
        self.status.set(Status::ZERO, (val & self.acc) == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
        self.status.set(Status::OVERFLOW, val & 0x40 != 0);
    }
}

impl Arithmetic for CPU {
    fn arith(&mut self, mode: AddressingMode, op: ArithOp, does_inc_cycle: bool) {
        let (val, inc_cycle) = self.get_absolute_addr(mode, self.pc).unwrap();
        let val = self.read(val);
        let val = match op {
            ArithOp::ADC => val,
            ArithOp::SBC => (val as i8).wrapping_neg().wrapping_sub(1) as u8,
        };

        let sum = (self.acc as u16) + (val as u16) + (self.status.contains(Status::CARRY) as u16);
        let carry = sum > 0xFF;
        let result = sum as u8;

        self.status.set(Status::CARRY, carry);
        self.status.set(Status::ZERO, result == 0);
        self.status.set(
            Status::OVERFLOW,
            (self.acc ^ result) & (val ^ result) & 0x80 != 0,
        );
        self.status.set(Status::NEGATIVE, result >> 7 == 1);

        self.acc = result;

        if inc_cycle && does_inc_cycle {
            self.cycles += 1;
        }
    }

    fn cmpr(&mut self, mode: AddressingMode, reg: Register) {
        let val = self.read(self.get_operand_addr(mode).unwrap());
        let register_val = match reg {
            Register::A => self.acc,
            Register::X => self.x,
            Register::Y => self.y,
            _ => panic!("Invalid register for compare"),
        };
        self.status.set(Status::CARRY, register_val >= val);
        self.status.set(Status::ZERO, register_val == val);
        self.status
            .set(Status::NEGATIVE, (register_val.wrapping_sub(val)) >> 7 == 1);
    }
}

impl IncDecOps for CPU {
    fn inc_dec(&mut self, mode: AddressingMode, op: IncDec) {
        let addr = self.get_operand_addr(mode).unwrap();
        let val = match op {
            IncDec::DEC => self.read(addr).wrapping_sub(1),
            IncDec::INC => self.read(addr).wrapping_add(1),
        };
        self.write(addr, val);
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    fn inc_dec_reg(&mut self, reg: Register, op: IncDec) {
        let val = match (reg, op) {
            (Register::X, IncDec::INC) => {
                self.x = self.x.wrapping_add(1);
                self.x
            }
            (Register::X, IncDec::DEC) => {
                self.x = self.x.wrapping_sub(1);
                self.x
            }
            (Register::Y, IncDec::INC) => {
                self.y = self.y.wrapping_add(1);
                self.y
            }
            (Register::Y, IncDec::DEC) => {
                self.y = self.y.wrapping_sub(1);
                self.y
            }
            _ => panic!("Invalid register for inc/dec"),
        };
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }
}

impl Shift for CPU {
    fn shift(&mut self, mode: AddressingMode, op: ShiftOp) {
        let addr = self.get_operand_addr(mode);
        let val = match mode {
            AddressingMode::Accumulator => self.acc,
            _ => self.read(addr.unwrap()),
        };
        let result = match op {
            ShiftOp::ASL => {
                self.status.set(Status::CARRY, val & 0x80 != 0);
                (val << 1) as u8
            }
            ShiftOp::LSR => {
                self.status.set(Status::CARRY, val & 0x01 != 0);
                (val >> 1) as u8
            }
        };
        match mode {
            AddressingMode::Accumulator => self.acc = result,
            _ => self.write(addr.unwrap(), result),
        };
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::NEGATIVE, result & 0x80 != 0);
    }

    fn rol(&mut self, mode: AddressingMode) {
        let result = match mode {
            AddressingMode::Accumulator => {
                let res = (self.acc << 1) | self.status.contains(Status::CARRY) as u8;
                self.status.set(Status::CARRY, self.acc & 0x80 != 0);
                self.acc = res;
                res
            }
            _ => {
                let addr = self.get_operand_addr(mode).unwrap();
                let val = self.read(addr);
                let res = (val << 1) | self.status.contains(Status::CARRY) as u8;
                self.status.set(Status::CARRY, val & 0x80 != 0);
                self.write(addr, res);
                res
            }
        };
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::NEGATIVE, (result >> 7) == 1);
    }

    fn ror(&mut self, mode: AddressingMode) {
        let result = match mode {
            AddressingMode::Accumulator => {
                let res = (self.acc >> 1) | (self.status.contains(Status::CARRY) as u8) << 7;
                self.status.set(Status::CARRY, self.acc & 0x01 == 1);
                self.acc = res;
                res
            }
            _ => {
                let addr = self.get_operand_addr(mode).unwrap();
                let val = self.read(addr);
                let res = (val >> 1) | (self.status.contains(Status::CARRY) as u8) << 7;
                self.status.set(Status::CARRY, val & 0x01 == 1);
                self.write(addr, res);
                res
            }
        };
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::NEGATIVE, (result >> 7) == 1);
    }
}

impl Jumps for CPU {
    fn jmp(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => self.pc = self.read_16(self.pc),
            _ => {
                // Emulate page boundary bug
                let addr = self.read_16(self.pc);
                self.pc = if addr & 0x00ff == 0x00ff {
                    (self.read(addr & 0xff00) as u16) << 8 | self.read(addr) as u16
                } else {
                    self.read_16(addr)
                };
            }
        }
    }

    fn jsr(&mut self) {
        self.stack_push_16(self.pc + 1);
        self.pc = self.read_16(self.pc);
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_16() + 1;
    }
}

impl Branches for CPU {
    fn branch(&mut self, flag: Status, set: bool) {
        if self.status.contains(flag) == set {
            self.cycles += 1;
            let jump = self.read(self.pc) as i8;
            let addr = self.pc.wrapping_add(1).wrapping_add(jump as u16);
            if self.pc.wrapping_add(1) & 0xff00 != addr & 0xff00 {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }
}

impl FlagChanges for CPU {
    fn flag(&mut self, flag: Status, set: bool) {
        self.status.set(flag, set);
    }
}

impl SysFuncs for CPU {
    fn nop(&mut self, mode: AddressingMode) {
        let inc_cycles = match self.get_absolute_addr(mode, self.pc) {
            Some((_, true)) => 1,
            _ => 0,
        };
        self.cycles += inc_cycles;
    }

    fn brk(&mut self) {
        self.pc += 1;
        if !self.status.contains(Status::INTERRUPT_DISABLE) {
            self.stack_push_16(self.pc);
            let mut flag = self.status.clone();
            flag.set(Status::BREAK, true);
            flag.set(Status::BREAK2, true);
            self.stack_push(flag.bits());
            self.status.insert(Status::INTERRUPT_DISABLE);
            self.pc = self.read_16(0xfffe);
        }
    }

    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(Status::BREAK);
        self.status.insert(Status::BREAK2);
        self.pc = self.stack_pop_16();
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            acc: 0,
            sp: 0xFD,
            pc: 0,
            status: Status { bits: 0x24 },
            memory: [0; 65536],
            cycles: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }

    pub fn read_16(&self, addr: u16) -> u16 {
        ((self.read(addr + 1) as u16) << 8) | (self.read(addr) as u16)
    }

    fn write_16(&mut self, addr: u16, val: u16) {
        self.write(addr, (val & 0xff) as u8);
        self.write(addr + 1, (val >> 8) as u8);
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

    pub fn load_prg_rom(&mut self, cartridge: File) {
        if cartridge.prg_rom_area.len() == 16384 {
            self.memory[0x8000..0xc000].copy_from_slice(&cartridge.prg_rom_area);
            self.memory[0xc000..0x10000].copy_from_slice(&cartridge.prg_rom_area);
        } else {
            self.memory[0x8000..0x10000].copy_from_slice(&cartridge.prg_rom_area);
        }
    }

    pub fn get_absolute_addr(&self, mode: AddressingMode, addr: u16) -> Option<(u16, bool)> {
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

    pub fn get_operand_addr(&self, mode: AddressingMode) -> Option<u16> {
        self.get_absolute_addr(mode, self.pc)
            .and_then(|(addr, _)| Some(addr))
    }

    pub fn reset(&mut self) {
        self.reset_with_val(self.read_16(0xFFFC));
    }

    pub fn reset_with_val(&mut self, val: u16) {
        self.pc = val;
        self.cycles = 7;
    }

    pub fn run(&mut self) {
        // Open log file for writing
        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("tests/nestest/log.txt")
            .unwrap();

        writeln!(log_file, "{}", trace(self)).unwrap();

        loop {
            let opcode = self.read(self.pc);
            let op = &OPS[OPS.binary_search_by_key(&opcode, |op| op.hex).unwrap()];

            self.pc += 1;

            if self.cycles >= 26554 {
                break;
            }

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

            self.cycles += op.cycles as u64;

            writeln!(log_file, "{}", trace(self)).unwrap();
        }
    }
}
