use crate::core::cpu::{Register, CPU};

use super::arithmetic::Arithmetic;

pub enum IncDec {
    INC,
    DEC,
}

pub(crate) trait IncDecOps: Arithmetic {
    fn inc_dec(&mut self, op: IncDec);

    fn isb(&mut self);

    fn inc(&mut self) {
        self.inc_dec(IncDec::INC)
    }

    fn inx(&mut self);

    fn iny(&mut self);

    fn dec(&mut self) {
        self.inc_dec(IncDec::DEC);
    }

    fn dex(&mut self);

    fn dey(&mut self);

    fn dcp(&mut self);
}

impl IncDecOps for CPU<'_> {
    fn inc_dec(&mut self, op: IncDec) {
        let addr = self.operand;
        let val = self.memory_read(addr);
        self.memory_write(addr, val); // Dummy write
        let val = match op {
            IncDec::DEC => val.wrapping_sub(1),
            IncDec::INC => val.wrapping_add(1),
        };
        self.set_zero_neg_flags(val);
        self.memory_write(addr, val);
    }

    fn isb(&mut self) {
        let mut val = self.get_operand_val();
        self.memory_write(self.operand, val);
        val = val.wrapping_add(1);
        self.add(val ^ 0xFF);
        self.memory_write(self.operand, val);
    }

    fn inx(&mut self) {
        self.set_register(Register::X, self.x.wrapping_add(1));
    }

    fn iny(&mut self) {
        self.set_register(Register::Y, self.y.wrapping_add(1));
    }

    fn dex(&mut self) {
        self.set_register(Register::X, self.x.wrapping_sub(1));
    }

    fn dey(&mut self) {
        self.set_register(Register::Y, self.y.wrapping_sub(1));
    }

    fn dcp(&mut self) {
        let mut val = self.get_operand_val();
        self.memory_write(self.operand, val);
        val = val.wrapping_sub(1);
        self.cmpr(self.acc, val);
        self.memory_write(self.operand, val);
    }
}
