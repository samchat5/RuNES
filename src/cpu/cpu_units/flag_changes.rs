use crate::cpu::{Status, CPU};

pub trait FlagChanges {
    fn flag(&mut self, flag: Status, set: bool);

    fn clc(&mut self) {
        self.flag(Status::CARRY, false);
    }

    fn cld(&mut self) {
        self.flag(Status::DECIMAL_MODE, false);
    }

    fn cli(&mut self) {
        self.flag(Status::INTERRUPT_DISABLE, false);
    }

    fn clv(&mut self) {
        self.flag(Status::OVERFLOW, false);
    }

    fn sec(&mut self) {
        self.flag(Status::CARRY, true);
    }

    fn sed(&mut self) {
        self.flag(Status::DECIMAL_MODE, true);
    }

    fn sei(&mut self) {
        self.flag(Status::INTERRUPT_DISABLE, true);
    }
}

impl FlagChanges for CPU<'_> {
    fn flag(&mut self, flag: Status, set: bool) {
        self.status.set(flag, set);
    }
}
