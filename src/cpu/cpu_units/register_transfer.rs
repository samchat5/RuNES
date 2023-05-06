use crate::cpu::cpu_units::load_store::LoadStore;
use crate::cpu::{Register, CPU};

pub trait RegisterTransfer {
    fn transfer(&mut self, from: Register, to: Register);

    fn lxa(&mut self);

    fn tax(&mut self) {
        self.transfer(Register::A, Register::X);
    }

    fn tay(&mut self) {
        self.transfer(Register::A, Register::Y);
    }

    fn txa(&mut self) {
        self.transfer(Register::X, Register::A);
    }

    fn tya(&mut self) {
        self.transfer(Register::Y, Register::A);
    }

    fn tsx(&mut self) {
        self.transfer(Register::P, Register::X);
    }

    fn txs(&mut self) {
        self.transfer(Register::X, Register::P);
    }
}

impl RegisterTransfer for CPU<'_> {
    fn transfer(&mut self, from: Register, to: Register) {
        let val_from = match from {
            Register::A => self.acc,
            Register::X => self.x,
            Register::Y => self.y,
            Register::P => self.sp,
        };
        match to {
            Register::P => self.sp = val_from,
            _ => self.set_register(to, val_from),
        };
    }

    fn lxa(&mut self) {
        self.lax();
        self.set_register(Register::A, self.acc);
    }
}
