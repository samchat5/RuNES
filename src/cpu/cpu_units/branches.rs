use crate::cpu::{Status, CPU};

pub trait Branches {
    fn branch(&mut self, flag: Status, set: bool);

    fn bcc(&mut self) {
        self.branch(Status::CARRY, false);
    }

    fn bcs(&mut self) {
        self.branch(Status::CARRY, true);
    }

    fn beq(&mut self) {
        self.branch(Status::ZERO, true);
    }

    fn bmi(&mut self) {
        self.branch(Status::NEGATIVE, true);
    }

    fn bne(&mut self) {
        self.branch(Status::ZERO, false);
    }

    fn bpl(&mut self) {
        self.branch(Status::NEGATIVE, false);
    }

    fn bvc(&mut self) {
        self.branch(Status::OVERFLOW, false);
    }

    fn bvs(&mut self) {
        self.branch(Status::OVERFLOW, true);
    }
}

impl Branches for CPU {
    fn branch(&mut self, flag: Status, set: bool) {
        if self.status.contains(flag) == set {
            self.cycles += 1;
            let jump = self.read(self.pc) as i8;
            let addr = self.pc.wrapping_add(1).wrapping_add(jump as u16);
            if self.pc.wrapping_add(1) & 0xff00 != addr & 0xff00 {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }
}
