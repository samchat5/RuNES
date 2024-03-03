use crate::core::cpu::{Register, Status, CPU};

pub trait StackOps {
    fn pha(&mut self);

    fn pla(&mut self);

    fn php(&mut self);

    fn plp(&mut self);
}

impl StackOps for CPU<'_> {
    fn pha(&mut self) {
        self.push(self.acc);
    }

    fn pla(&mut self) {
        self.dummy_read();
        let popped = self.pop();
        self.set_register(Register::A, popped);
    }

    fn php(&mut self) {
        let flags = self.status.bits() | Status::BREAK.bits() | Status::BREAK2.bits();
        self.push(flags);
    }

    fn plp(&mut self) {
        self.dummy_read();
        self.status = Status::from_bits_truncate(self.pop());
    }
}
