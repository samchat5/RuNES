use crate::cpu::{AddressingMode, Status, CPU};

use super::{arithmetic::Arithmetic, logical::Logical};

pub enum ShiftOp {
    ASL,
    LSR,
}

pub(crate) trait Shift: Logical + Arithmetic {
    fn shift(&mut self, mode: AddressingMode, op: ShiftOp);

    fn rol(&mut self, mode: AddressingMode);

    fn ror(&mut self, mode: AddressingMode);

    fn asl(&mut self, mode: AddressingMode) {
        self.shift(mode, ShiftOp::ASL);
    }

    fn lsr(&mut self, mode: AddressingMode) {
        self.shift(mode, ShiftOp::LSR);
    }

    fn slo(&mut self, mode: AddressingMode) {
        self.asl(mode);
        self.ora_no_inc(mode);
    }

    fn rla(&mut self, mode: AddressingMode) {
        self.rol(mode);
        self.and_no_inc(mode);
    }

    fn sre(&mut self, mode: AddressingMode) {
        self.lsr(mode);
        self.eor_no_inc(mode);
    }

    fn rra(&mut self, mode: AddressingMode) {
        self.ror(mode);
        self.adc_no_inc(mode);
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
