use crate::cpu::{AddressingMode, Register, Status, CPU};

pub enum ArithOp {
    ADC,
    SBC,
}

pub trait Arithmetic {
    fn arith(&mut self, mode: AddressingMode, op: ArithOp, does_inc_cycle: bool);

    fn cmpr(&mut self, mode: AddressingMode, reg: Register);

    fn adc(&mut self, mode: AddressingMode) {
        self.arith(mode, ArithOp::ADC, true);
    }

    fn adc_no_inc(&mut self, mode: AddressingMode) {
        self.arith(mode, ArithOp::ADC, false);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        self.arith(mode, ArithOp::SBC, true);
    }

    fn sbc_no_inc(&mut self, mode: AddressingMode) {
        self.arith(mode, ArithOp::SBC, false);
    }

    fn cmp(&mut self, mode: AddressingMode) {
        self.cmpr(mode, Register::A);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        self.cmpr(mode, Register::X);
    }

    fn cpy(&mut self, mode: AddressingMode) {
        self.cmpr(mode, Register::Y);
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
        let addr = self.get_operand_addr(mode).unwrap();
        let val = self.read(addr);
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
