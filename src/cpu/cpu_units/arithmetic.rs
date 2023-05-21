use crate::cpu::{Register, Status, CPU};

pub enum ArithOp {
    ADC,
    SBC,
}

pub trait Arithmetic {
    fn add(&mut self, val: u8);

    fn add_from_operand(&mut self, op: ArithOp);

    fn cmpr(&mut self, register_val: u8, val: u8);

    fn cmpr_from_operand(&mut self, reg: Register);

    fn axs(&mut self);

    fn adc(&mut self) {
        self.add_from_operand(ArithOp::ADC);
    }

    fn sbc(&mut self) {
        self.add_from_operand(ArithOp::SBC);
    }

    fn cmp(&mut self) {
        self.cmpr_from_operand(Register::A);
    }

    fn cpx(&mut self) {
        self.cmpr_from_operand(Register::X);
    }

    fn cpy(&mut self) {
        self.cmpr_from_operand(Register::Y);
    }
}

impl Arithmetic for CPU<'_> {
    fn add(&mut self, val: u8) {
        let res = self.acc as u16 + val as u16 + u16::from(self.status.contains(Status::CARRY));
        self.set_zero_neg_flags(val);
        self.status.set(Status::CARRY, res > 0xFF);
        self.status.set(
            Status::OVERFLOW,
            (!(self.acc ^ val) & (self.acc ^ res as u8) & 0x80) != 0,
        );
        self.set_register(Register::A, res as u8);
    }

    fn add_from_operand(&mut self, op: ArithOp) {
        let operand_val = self.get_operand_val();
        match op {
            ArithOp::ADC => self.add(operand_val),
            ArithOp::SBC => self.add(operand_val ^ 0xFF),
        }
    }

    fn cmpr(&mut self, register_val: u8, val: u8) {
        self.status.set(Status::CARRY, register_val >= val);
        self.status.set(Status::ZERO, register_val == val);
        self.status.set(
            Status::NEGATIVE,
            (register_val.wrapping_sub(val) & 0x80) == 0x80,
        );
    }

    fn cmpr_from_operand(&mut self, reg: Register) {
        let operand_val = self.get_operand_val();
        match reg {
            Register::A => self.cmpr(self.acc, operand_val),
            Register::X => self.cmpr(self.x, operand_val),
            Register::Y => self.cmpr(self.y, operand_val),
            _ => unreachable!(),
        }
    }

    fn axs(&mut self) {
        let op_val = self.get_operand_val();
        let val = (self.acc & self.x).wrapping_sub(op_val);
        self.status.set(Status::CARRY, self.acc & self.x >= op_val);
        self.set_register(Register::X, val);
    }
}
