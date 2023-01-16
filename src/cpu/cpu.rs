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

impl LoadStore for CPU {
    fn ld(&mut self, mode: AddressingMode, reg: Register) {
        let (val, inc_cycles) = self.get_absolute_addr(mode, self.pc).unwrap();
        if inc_cycles {
            self.cycles += 1;
        }
        let val = self.read(val);
        match reg {
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::A => self.acc = val,
            _ => panic!("Invalid register for load"),
        }
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    fn st(&mut self, mode: AddressingMode, reg: Register) {
        let addr = self.get_operand_addr(mode).unwrap();
        match reg {
            Register::X => self.write(addr, self.x),
            Register::Y => self.write(addr, self.y),
            Register::A => self.write(addr, self.acc),
            _ => panic!("Invalid register for store"),
        }
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
    fn bit_op(&mut self, mode: AddressingMode, op: LogicalOp) {
        let val = self.read(self.get_operand_addr(mode).unwrap());
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
    fn arith(&mut self, mode: AddressingMode, op: ArithOp) {
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

        if inc_cycle {
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
        match op {
            ShiftOp::ASL => self.status.set(Status::CARRY, self.acc & 0x80 != 0),
            ShiftOp::LSR => self.status.set(Status::CARRY, self.acc & 0x01 != 0),
        }
        let result = match mode {
            AddressingMode::Accumulator => {
                let res = match op {
                    ShiftOp::ASL => (self.acc << 1) as u8,
                    ShiftOp::LSR => self.acc >> 1,
                };
                self.acc = res;
                res
            }
            _ => {
                let addr = self.get_operand_addr(mode).unwrap();
                let val = self.read(addr);
                let res = match op {
                    ShiftOp::ASL => ((val as i8) << 1) as u8,
                    ShiftOp::LSR => val >> 1,
                };
                self.write(addr, res);
                res
            }
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
                self.status.set(Status::CARRY, val & 0x80 != 0);
                let res = (val << 1) | self.status.contains(Status::CARRY) as u8;
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

    pub fn run(&mut self) {
        // Set PC to interrupt vector
        self.pc = 0xc000;
        self.cycles = 7;

        // Open log file for writing
        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open("log.txt")
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

            match opcode {
                0x00 => self.brk(),
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => {
                    self.ora(op.addressing_mode);
                }
                0x0a | 0x06 | 0x16 | 0x0e | 0x1e => self.asl(op.addressing_mode),
                0x08 => self.php(),
                0x10 => self.bpl(),
                0x18 => self.clc(),
                0x20 => self.jsr(),
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => {
                    self.and(op.addressing_mode)
                }
                0x24 | 0x2c => self.bit(op.addressing_mode),
                0x2a | 0x26 | 0x36 | 0x2e | 0x3e => self.rol(op.addressing_mode),
                0x28 => self.plp(),
                0x30 => self.bmi(),
                0x38 => self.sec(),
                0x40 => self.rti(),
                0x49 | 0x45 | 0x55 | 0x4D | 0x5d | 0x59 | 0x41 | 0x51 => {
                    self.eor(op.addressing_mode)
                }
                0x4a | 0x46 | 0x56 | 0x4e | 0x5e => self.lsr(op.addressing_mode),
                0x48 => self.pha(),
                0x4c | 0x6c => self.jmp(op.addressing_mode),
                0x50 => self.bvc(),
                0x58 => self.cli(),
                0x60 => self.rts(),
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                    self.adc(op.addressing_mode)
                }
                0x6a | 0x66 | 0x76 | 0x6e | 0x7e => self.ror(op.addressing_mode),
                0x68 => self.pla(),
                0x70 => self.bvs(),
                0x78 => self.sei(),
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.sta(op.addressing_mode),
                0x84 | 0x94 | 0x8c => self.sty(op.addressing_mode),
                0x86 | 0x96 | 0x8e => self.stx(op.addressing_mode),
                0x88 => self.dey(),
                0x8a => self.txa(),
                0x90 => self.bcc(),
                0x98 => self.tya(),
                0x9a => self.txs(),
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.ldy(op.addressing_mode),
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(op.addressing_mode)
                }
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.ldx(op.addressing_mode),
                0xa8 => self.tay(),
                0xaa => self.tax(),
                0xb0 => self.bcs(),
                0xb8 => self.clv(),
                0xba => self.tsx(),
                0xc0 | 0xc4 | 0xcc => self.cpy(op.addressing_mode),
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => {
                    self.cmp(op.addressing_mode)
                }
                0xc6 | 0xd6 | 0xce | 0xde => self.dec(op.addressing_mode),
                0xc8 => self.iny(),
                0xca => self.dex(),
                0xd0 => self.bne(),
                0xd8 => self.cld(),
                0xe0 | 0xe4 | 0xec => self.cpx(op.addressing_mode),
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => {
                    self.sbc(op.addressing_mode)
                }
                0xe6 | 0xf6 | 0xee | 0xfe => self.inc(op.addressing_mode),
                0xe8 => self.inx(),
                0xea => self.nop(),
                0xf0 => self.beq(),
                0xf8 => self.sed(),
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
