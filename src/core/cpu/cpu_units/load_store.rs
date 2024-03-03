use crate::core::cpu::{Register, CPU};

pub trait LoadStore {
    fn ld(&mut self, reg: Register);

    fn st(&mut self, reg: Register);

    fn lax(&mut self);

    fn lda(&mut self) {
        self.ld(Register::A);
    }

    fn ldx(&mut self) {
        self.ld(Register::X);
    }

    fn ldy(&mut self) {
        self.ld(Register::Y);
    }

    fn sax(&mut self);

    fn sta(&mut self) {
        self.st(Register::A);
    }

    fn stx(&mut self) {
        self.st(Register::X);
    }

    fn sty(&mut self) {
        self.st(Register::Y);
    }
}

impl LoadStore for CPU<'_> {
    fn ld(&mut self, reg: Register) {
        let val = self.get_operand_val();
        self.set_register(reg, val);
    }

    fn st(&mut self, reg: Register) {
        self.memory_write(
            self.operand,
            match reg {
                Register::A => self.acc,
                Register::X => self.x,
                Register::Y => self.y,
                _ => unreachable!(),
            },
        );
    }

    fn lax(&mut self) {
        let val = self.get_operand_val();
        self.set_register(Register::X, val);
        self.set_register(Register::A, val);
    }

    fn sax(&mut self) {
        self.memory_write(self.operand, self.acc & self.x);
    }
}
