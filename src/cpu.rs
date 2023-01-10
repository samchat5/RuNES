use core::panic;
use std::ops::Add;

use bitflags::bitflags;

bitflags! {
    struct Status: u8 {
        const CARRY = 0x01;
        const ZERO = 0x02;
        const INTERRUPT_DISABLE = 0x04;
        const DECIMAL_MODE = 0x08;
        const BREAK = 0x10;
        const BREAK2 = 0x20;
        const OVERFLOW = 0x40;
        const NEGATIVE = 0x80;
        const STATUS = Self::CARRY.bits | Self::ZERO.bits | Self::INTERRUPT_DISABLE.bits | Self::DECIMAL_MODE.bits | Self::BREAK.bits | Self::OVERFLOW.bits | Self::NEGATIVE.bits;
    }
}

const STACK_BASE: u16 = 0x0100;

enum ShiftOp {
    LSR,
    ASL,
}

enum IncDec {
    INC,
    DEC,
}

enum ArithOp {
    ADC,
    SBC,
}

enum LogicalOp {
    EOR,
    AND,
    ORA,
}

enum Register {
    X,
    Y,
    A,
    P,
}

enum AddressingMode {
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

struct CPU {
    // Registers
    x: u8,
    y: u8,
    acc: u8,
    sp: u8,
    pc: u16,

    // Status flags
    status: Status,

    // Memory
    memory: [u8; 0xffff],
}

impl CPU {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            acc: 0,
            sp: 0xFD,
            pc: 0,
            status: Status { bits: 0x34 },
            memory: [0; 0xffff],
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }

    fn read_16(&self, addr: u16) -> u16 {
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

    fn get_operand_addr(&self, mode: AddressingMode) -> Option<u16> {
        match mode {
            AddressingMode::Immediate => Some(self.pc),
            AddressingMode::ZeroPage => Some(self.read(self.pc as u16) as u16),
            AddressingMode::ZeroPageX => Some(self.read(self.pc).wrapping_add(self.x) as u16),
            AddressingMode::ZeroPageY => Some(self.read(self.pc).wrapping_add(self.y) as u16),
            AddressingMode::Absolute => Some(self.read_16(self.pc) as u16),
            AddressingMode::AbsoluteX => Some(self.read_16(self.pc).wrapping_add(self.x as u16)),
            AddressingMode::AbsoluteY => Some(self.read_16(self.pc).wrapping_add(self.y as u16)),
            AddressingMode::IndexedIndirect => Some(
                ((self.read(self.pc).wrapping_add(self.x + 1) as u16) << 8)
                    | (self.read(self.pc).wrapping_add(self.x) as u16),
            ),
            AddressingMode::IndirectIndexed => Some(self.read_16(
                ((self.read(self.pc.wrapping_add(1)) as u16) << 8)
                    | (self.read(self.pc) as u16).wrapping_add(self.y as u16),
            )),
            _ => None,
        }
    }

    // ----------------------------------- Load/Store Operations -----------------------------------

    fn ld(&mut self, mode: AddressingMode, reg: Register) {
        let val = self.read(self.get_operand_addr(mode).unwrap());
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

    // ------------------------------------- Register Transfers ------------------------------------

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
        self.status.set(Status::ZERO, self.x == 0);
        self.status.set(Status::NEGATIVE, self.x & 0x80 != 0);
    }

    // -------------------------------------- Stack Operations -------------------------------------

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
            Register::P => self.status.bits,
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
            Register::P => self.status.bits = self.stack_pop(),
            _ => panic!("Invalid register for pull"),
        }
    }

    // ------------------------------------------ Logical ------------------------------------------

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

    // ----------------------------------------- Arithmetic ----------------------------------------

    fn arith(&mut self, mode: AddressingMode, op: ArithOp) {
        let val = self.read(self.get_operand_addr(mode).unwrap());
        let (result, carry) = match op {
            ArithOp::ADC => self
                .acc
                .overflowing_add(val)
                .0
                .overflowing_add(self.status.contains(Status::CARRY) as u8),
            ArithOp::SBC => self
                .acc
                .overflowing_sub(val)
                .0
                .overflowing_sub(!self.status.contains(Status::CARRY) as u8),
        };
        self.status.set(Status::CARRY, carry);
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::NEGATIVE, result & 0x80 != 0);
        self.status.set(
            Status::OVERFLOW,
            (self.acc ^ result) & (val ^ result) & 0x80 != 0,
        );
        self.acc = result;
    }

    fn cmp(&mut self, mode: AddressingMode, reg: Register) {
        let val = self.read(self.get_operand_addr(mode).unwrap());
        let (result, carry) = match reg {
            Register::A => self.acc,
            Register::X => self.x,
            Register::Y => self.y,
            _ => panic!("Invalid register for compare"),
        }
        .overflowing_sub(val);
        self.status.set(Status::CARRY, carry);
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::NEGATIVE, result & 0x80 != 0);
    }

    // ---------------------------------- Increments & Decrements ----------------------------------

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
        let inc_dec = match op {
            IncDec::DEC => -1,
            IncDec::INC => 1,
        };
        let val = match reg {
            Register::X => {
                self.x = (self.x as i8 + inc_dec) as u8;
                self.x
            }
            Register::Y => {
                self.y += (self.y as i8 + inc_dec) as u8;
                self.y
            }
            _ => panic!("Invalid register for increment"),
        };
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    // ------------------------------------------- Shifts ------------------------------------------

    fn shift(&mut self, mode: AddressingMode, op: ShiftOp) {
        let addr = self.get_operand_addr(mode).unwrap();
        let val = self.read(addr);
        self.status.set(Status::CARRY, val & 0x80 != 0);
        let result = match op {
            ShiftOp::ASL => ((val as i8) << 1) as u8,
            ShiftOp::LSR => val >> 1,
        };
        self.write(addr, result);
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::NEGATIVE, result & 0x80 != 0);
    }

    fn rol(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => {
                self.status.set(Status::CARRY, self.acc & 0x80 != 0);
                self.acc = (self.acc << 1) | self.status.contains(Status::CARRY) as u8;
            }
            _ => {
                let addr = self.get_operand_addr(mode).unwrap();
                let val = self.read(addr);
                self.status.set(Status::CARRY, val & 0x80 != 0);
                self.write(addr, (val << 1) | self.status.contains(Status::CARRY) as u8);
            }
        }
    }

    fn ror(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => {
                self.status.set(Status::CARRY, self.acc & 0x01 != 0);
                self.acc = (self.acc >> 1) | (self.status.contains(Status::CARRY) as u8) << 7;
            }
            _ => {
                let addr = self.get_operand_addr(mode).unwrap();
                let val = self.read(addr);
                self.status.set(Status::CARRY, val & 0x01 == 0);
                self.write(
                    addr,
                    (val >> 1) | (self.status.contains(Status::CARRY) as u8) << 7,
                )
            }
        }
    }

    // ------------------------------------------- Jumps -------------------------------------------

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

    // ------------------------------------------ Branches -----------------------------------------

    fn branch(&mut self, flag: Status, set: bool) {
        if self.status.contains(flag) == set {
            self.pc = self
                .pc
                .wrapping_add(1)
                .wrapping_add(self.read(self.pc) as i8 as u16);
        }
    }

    // ------------------------------------ Status Flag Changes ------------------------------------

    fn flag(&mut self, flag: Status, set: bool) {
        self.status.set(flag, set);
    }

    // -------------------------------------- System Functions -------------------------------------

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

    fn nop(&mut self) {}

    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(Status::BREAK);
        self.status.insert(Status::BREAK2);
        self.pc = self.stack_pop_16();
    }
}
