use crate::cpu::{AddressingMode, CPU};

pub trait Jumps {
    fn jmp_to_addr(&mut self, addr: u16);

    fn jmp(&mut self);

    fn jsr(&mut self);

    fn rts(&mut self);
}

impl Jumps for CPU<'_> {
    fn jmp_to_addr(&mut self, addr: u16) {
        self.pc = addr
    }

    fn jmp(&mut self) {
        match self.instr_addr_mode {
            AddressingMode::Absolute => {
                self.jmp_to_addr(self.operand);
            }
            AddressingMode::Indirect => {
                let val = self.get_ind();
                self.jmp_to_addr(val)
            }
            _ => unreachable!(),
        }
    }

    fn jsr(&mut self) {
        let addr = self.operand;
        self.dummy_read();
        self.push_word(self.pc - 1);
        self.jmp_to_addr(addr);
    }

    fn rts(&mut self) {
        let addr = self.pop_word();
        self.dummy_read();
        self.dummy_read();
        self.pc = addr + 1;
    }
}
