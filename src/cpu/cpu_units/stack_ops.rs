use crate::cpu::{Register, Status, CPU};

pub trait StackOps {
    fn ph(&mut self, reg: Register);

    fn pl(&mut self, reg: Register);

    fn tsx(&mut self);

    fn txs(&mut self);

    fn pha(&mut self) {
        self.ph(Register::A);
    }

    fn pla(&mut self) {
        self.pl(Register::A);
    }

    fn php(&mut self) {
        self.ph(Register::P);
    }

    fn plp(&mut self) {
        self.pl(Register::P);
    }
}

impl StackOps for CPU {
    fn ph(&mut self, reg: Register) {
        self.stack_push(match reg {
            Register::A => self.acc,
            Register::P => (self.status | Status::BREAK | Status::BREAK2).bits,
            _ => panic!("Invalid register for push"),
        });
    }

    fn pl(&mut self, reg: Register) {
        match reg {
            Register::A => {
                self.acc = self.stack_pop();
                self.status.set(Status::ZERO, self.acc == 0);
                self.status.set(Status::NEGATIVE, self.acc & 0x80 != 0);
            }
            Register::P => {
                self.status.bits = self.stack_pop() & !Status::BREAK.bits | Status::BREAK2.bits
            }
            _ => panic!("Invalid register for pull"),
        }
    }

    fn tsx(&mut self) {
        self.x = self.sp;
        self.status.set(Status::ZERO, self.x == 0);
        self.status.set(Status::NEGATIVE, self.x & 0x80 != 0);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }
}
