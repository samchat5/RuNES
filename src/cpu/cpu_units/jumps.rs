use crate::cpu::{AddressingMode, CPU};

pub trait Jumps {
    fn jmp(&mut self, mode: AddressingMode);

    fn jsr(&mut self);

    fn rts(&mut self);
}

impl Jumps for CPU {
    fn jmp(&mut self, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => self.pc = self.read_16(self.pc),
            _ => {
                // Emulate page boundary bug
                let addr = self.read_16(self.pc);
                self.pc = if addr & 0x00ff == 0x00ff {
                    (self.read(addr & 0xff00) as u16) << 8 | self.read(addr) as u16
                } else {
                    self.read_16(addr)
                };
            }
        }
    }

    fn jsr(&mut self) {
        self.stack_push_16(self.pc + 1);
        self.pc = self.read_16(self.pc);
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_16() + 1;
    }
}
