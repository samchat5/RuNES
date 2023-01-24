use crate::cpu::{AddressingMode, Register, Status, CPU};

pub trait LoadStore {
    fn ld(&mut self, mode: AddressingMode, reg: Vec<Register>);

    fn st(&mut self, mode: AddressingMode, reg: Vec<Register>);

    fn lax(&mut self, mode: AddressingMode) {
        self.ld(mode, vec![Register::A, Register::X]);
    }

    fn lda(&mut self, mode: AddressingMode) {
        self.ld(mode, vec![Register::A]);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        self.ld(mode, vec![Register::X]);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        self.ld(mode, vec![Register::Y]);
    }

    fn sax(&mut self, mode: AddressingMode) {
        self.st(mode, vec![Register::A, Register::X]);
    }

    fn sta(&mut self, mode: AddressingMode) {
        self.st(mode, vec![Register::A]);
    }

    fn stx(&mut self, mode: AddressingMode) {
        self.st(mode, vec![Register::X]);
    }

    fn sty(&mut self, mode: AddressingMode) {
        self.st(mode, vec![Register::Y]);
    }
}

impl LoadStore for CPU {
    fn ld(&mut self, mode: AddressingMode, regs: Vec<Register>) {
        let (val, inc_cycles) = self.get_absolute_addr(mode, self.pc).unwrap();
        if inc_cycles {
            self.cycles += 1;
        }
        let val = self.read(val);
        regs.iter().for_each(|reg| match reg {
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::A => self.acc = val,
            _ => panic!("Invalid register for load"),
        });
        self.status.set(Status::ZERO, val == 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    fn st(&mut self, mode: AddressingMode, regs: Vec<Register>) {
        let addr = self.get_operand_addr(mode).unwrap();
        self.write(
            addr,
            regs.iter().fold(0xff, |acc, r| {
                acc & match r {
                    Register::X => self.x,
                    Register::Y => self.y,
                    Register::A => self.acc,
                    _ => panic!("Invalid register for store"),
                }
            }),
        );
    }
}
