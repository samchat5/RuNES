use crate::cpu::{Register, Status, CPU};

pub trait RegisterTransfer {
    fn transfer(&mut self, from: Register, to: Register);

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
}

impl RegisterTransfer for CPU<'_> {
    fn transfer(&mut self, from: Register, to: Register) {
        match (from, to) {
            (Register::A, Register::X) => {
                self.x = self.acc;
            }
            (Register::A, Register::Y) => {
                self.y = self.acc;
            }
            (Register::X, Register::A) => {
                self.acc = self.x;
            }
            (Register::Y, Register::A) => {
                self.acc = self.y;
            }
            _ => panic!("Invalid transfer"),
        }
        self.status.set(Status::ZERO, self.acc == 0);
        self.status.set(Status::NEGATIVE, self.acc >> 7 == 1);
    }
}
