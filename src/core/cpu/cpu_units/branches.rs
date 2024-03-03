use crate::core::cpu::{Status, CPU};

pub trait Branches {
    fn branch_relative(&mut self, flag: Status, set: bool);

    fn bcc(&mut self) {
        self.branch_relative(Status::CARRY, false);
    }

    fn bcs(&mut self) {
        self.branch_relative(Status::CARRY, true);
    }

    fn beq(&mut self) {
        self.branch_relative(Status::ZERO, true);
    }

    fn bmi(&mut self) {
        self.branch_relative(Status::NEGATIVE, true);
    }

    fn bne(&mut self) {
        self.branch_relative(Status::ZERO, false);
    }

    fn bpl(&mut self) {
        self.branch_relative(Status::NEGATIVE, false);
    }

    fn bvc(&mut self) {
        self.branch_relative(Status::OVERFLOW, false);
    }

    fn bvs(&mut self) {
        self.branch_relative(Status::OVERFLOW, true);
    }
}

impl Branches for CPU<'_> {
    fn branch_relative(&mut self, flag: Status, set: bool) {
        let branch = if set {
            self.status.contains(flag)
        } else {
            !self.status.contains(flag)
        };
        let offset = self.operand as i8;
        if branch {
            if self.run_irq && !self.prev_run_irq {
                self.run_irq = false;
            }
            self.dummy_read();
            if Self::check_page_crossed_i8(self.pc, offset) {
                self.dummy_read();
            }
            self.pc = match offset < 0 {
                true => self.pc.wrapping_sub(offset.unsigned_abs() as u16),
                false => self.pc.wrapping_add(offset as u16),
            };
        }
    }
}
