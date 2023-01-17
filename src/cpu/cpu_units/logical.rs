use crate::cpu::{AddressingMode, Status, CPU};

pub enum LogicalOp {
    EOR,
    AND,
    ORA,
}

pub trait Logical {
    fn bit_op(&mut self, mode: AddressingMode, op: LogicalOp, does_inc_cycle: bool);

    fn bit(&mut self, mode: AddressingMode);

    fn and(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::AND, true);
    }

    fn and_no_inc(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::AND, false);
    }

    fn ora(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::ORA, true);
    }

    fn ora_no_inc(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::ORA, false);
    }

    fn eor(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::EOR, true);
    }

    fn eor_no_inc(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::EOR, false);
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
