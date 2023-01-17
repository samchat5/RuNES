use crate::cpu::{AddressingMode, Register, Status, CPU};

use super::arithmetic::Arithmetic;

pub enum IncDec {
    INC,
    DEC,
}

pub(crate) trait IncDecOps: Arithmetic {
    fn inc_dec(&mut self, mode: AddressingMode, op: IncDec);

    fn inc_dec_reg(&mut self, reg: Register, op: IncDec);

    fn isb(&mut self, mode: AddressingMode) {
        self.inc(mode);
        self.sbc_no_inc(mode);
    }

    fn inc(&mut self, mode: AddressingMode) {
        self.inc_dec(mode, IncDec::INC);
    }

    fn inx(&mut self) {
        self.inc_dec_reg(Register::X, IncDec::INC);
    }

    fn iny(&mut self) {
        self.inc_dec_reg(Register::Y, IncDec::INC);
    }

    fn dec(&mut self, mode: AddressingMode) {
        self.inc_dec(mode, IncDec::DEC);
    }

    fn dex(&mut self) {
        self.inc_dec_reg(Register::X, IncDec::DEC);
    }

    fn dey(&mut self) {
        self.inc_dec_reg(Register::Y, IncDec::DEC);
    }

    fn dcp(&mut self, mode: AddressingMode) {
        self.dec(mode);
        self.cmp(mode);
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
