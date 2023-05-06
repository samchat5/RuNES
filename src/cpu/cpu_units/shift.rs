use crate::cpu::{AddressingMode, Register, Status, CPU};

use super::{arithmetic::Arithmetic, logical::Logical};

pub enum ShiftOp {
    ASL,
    LSR,
    ROL,
    ROR,
}

pub(crate) trait Shift: Logical + Arithmetic {
    fn shift(&mut self, val: u8, op: ShiftOp) -> u8;

    fn sh(&mut self, reg: Register);

    fn get_shift_val(&mut self, op: ShiftOp);

    fn rol(&mut self) {
        self.get_shift_val(ShiftOp::ROL);
    }

    fn ror(&mut self) {
        self.get_shift_val(ShiftOp::ROR);
    }

    fn asl(&mut self) {
        self.get_shift_val(ShiftOp::ASL)
    }

    fn lsr(&mut self) {
        self.get_shift_val(ShiftOp::LSR)
    }

    fn slo(&mut self);

    fn rla(&mut self);

    fn sre(&mut self);

    fn rra(&mut self);

    fn shy(&mut self) {
        self.sh(Register::Y);
    }

    fn shx(&mut self) {
        self.sh(Register::X);
    }
}

impl Shift for CPU<'_> {
    fn shift(&mut self, val: u8, op: ShiftOp) -> u8 {
        let res = match op {
            ShiftOp::ASL => {
                self.status.set(Status::CARRY, val & 0x80 != 0);
                val << 1
            }
            ShiftOp::LSR => {
                self.status.set(Status::CARRY, val & 0x01 != 0);
                val >> 1
            }
            ShiftOp::ROL => {
                let carry = self.status.contains(Status::CARRY);
                self.status.set(Status::CARRY, val & 0x80 != 0);
                (val << 1) | (carry as u8)
            }
            ShiftOp::ROR => {
                let carry = self.status.contains(Status::CARRY);
                self.status.set(Status::CARRY, val & 0x01 != 0);
                (val >> 1) | ((carry as u8) << 7)
            }
        };
        self.set_zero_neg_flags(res);
        res
    }

    fn get_shift_val(&mut self, op: ShiftOp) {
        match self.instr_addr_mode {
            AddressingMode::Accumulator => {
                self.acc = self.shift(self.acc, op);
            }
            _ => {
                let addr = self.operand;
                let val = self.memory_read(addr);
                self.memory_write(addr, val); // Dummy write
                let shifted = self.shift(val, op);
                self.memory_write(addr, shifted);
            }
        }
    }

    fn slo(&mut self) {
        let val = self.get_operand_val();
        self.memory_write(self.operand, val);
        let shifted = self.shift(val, ShiftOp::ASL);
        self.set_register(Register::A, self.acc | shifted);
        self.memory_write(self.operand, shifted);
    }

    fn rla(&mut self) {
        let val = self.get_operand_val();
        self.memory_write(self.operand, val);
        let shifted = self.shift(val, ShiftOp::ROL);
        self.set_register(Register::A, self.acc & shifted);
        self.memory_write(self.operand, shifted);
    }

    fn sre(&mut self) {
        let val = self.get_operand_val();
        self.memory_write(self.operand, val);
        let shifted = self.shift(val, ShiftOp::LSR);
        self.set_register(Register::A, self.acc ^ shifted);
        self.memory_write(self.operand, shifted);
    }

    fn rra(&mut self) {
        let val = self.get_operand_val();
        self.memory_write(self.operand, val);
        let shifted = self.shift(val, ShiftOp::ROR);
        self.add(shifted);
        self.memory_write(self.operand, shifted);
    }

    fn sh(&mut self, reg: Register) {
        let reg = match reg {
            Register::X => self.x,
            Register::Y => self.y,
            _ => panic!("Invalid register"),
        };
        let addr_hi = (self.operand >> 8) as u8;
        let addr_lo = (self.operand & 0xFF) as u8;
        let val = reg & (addr_hi + 1);
        self.memory_write(((val as u16) << 8) | addr_lo as u16, val);
    }
}
