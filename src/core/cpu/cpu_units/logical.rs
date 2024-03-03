use crate::core::cpu::{Register, Status, CPU};

pub enum LogicalOp {
    EOR,
    AND,
    ORA,
}

pub trait Logical {
    fn bit_op(&mut self, op: LogicalOp);

    fn bit(&mut self);

    fn anc(&mut self);

    fn asr(&mut self);

    fn arr(&mut self);

    fn and(&mut self) {
        self.bit_op(LogicalOp::AND);
    }

    fn ora(&mut self) {
        self.bit_op(LogicalOp::ORA);
    }

    fn eor(&mut self) {
        self.bit_op(LogicalOp::EOR);
    }
}

impl Logical for CPU<'_> {
    fn bit_op(&mut self, op: LogicalOp) {
        let val = self.get_operand_val();
        self.set_register(
            Register::A,
            match op {
                LogicalOp::AND => self.acc & val,
                LogicalOp::ORA => self.acc | val,
                LogicalOp::EOR => self.acc ^ val,
            },
        );
    }

    fn bit(&mut self) {
        let val = self.get_operand_val();
        self.status.set(Status::ZERO, self.acc & val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
        self.status.set(Status::OVERFLOW, val & 0x40 != 0);
    }

    fn anc(&mut self) {
        let op_val = self.get_operand_val();
        self.set_register(Register::A, self.acc & op_val);
        self.status
            .set(Status::CARRY, self.status.contains(Status::NEGATIVE));
    }

    fn asr(&mut self) {
        let op_val = self.get_operand_val();
        self.set_register(Register::A, self.acc & op_val);
        self.status.set(Status::CARRY, self.acc & 0x01 != 0);
        self.set_register(Register::A, self.acc >> 1);
    }

    fn arr(&mut self) {
        let op_val = self.get_operand_val();
        let and = self.acc & op_val;
        let shift = and >> 1;
        let carry = (self.status.contains(Status::CARRY) as u8) * 0x80;
        self.set_register(Register::A, shift | carry);
        self.status.set(Status::CARRY, self.acc & 0x40 != 0);
        self.status.set(
            Status::OVERFLOW,
            (self.status.contains(Status::CARRY) as u8) ^ ((self.acc >> 5) & 0x01) != 0,
        );
    }
}
