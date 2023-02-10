use crate::cpu::{AddressingMode, Status, CPU};

pub trait SysFuncs {
    fn nop(&mut self, mode: AddressingMode);

    fn brk(&mut self);

    fn rti(&mut self);

    fn nmi(&mut self);
}

impl SysFuncs for CPU {
    fn nop(&mut self, mode: AddressingMode) {
        let inc_cycles = match self.get_absolute_addr(mode, self.pc) {
            Some((_, true)) => 1,
            _ => 0,
        };
        self.bus.tick(inc_cycles);
    }

    fn brk(&mut self) {
        self.pc += 1;
        if !self.status.contains(Status::INTERRUPT_DISABLE) {
            self.stack_push_16(self.pc);
            let mut flag = self.status;
            flag.set(Status::BREAK, true);
            flag.set(Status::BREAK2, true);
            self.stack_push(flag.bits());
            self.status.insert(Status::INTERRUPT_DISABLE);
            self.pc = self.read_16(0xfffe);
        }
    }

    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(Status::BREAK);
        self.status.insert(Status::BREAK2);
        self.pc = self.stack_pop_16();
    }

    fn nmi(&mut self) {
        self.stack_push_16(self.pc);
        let mut status = self.status;
        status.set(Status::BREAK, false);
        status.set(Status::BREAK2, true);
        self.stack_push(status.bits);
        self.status.insert(Status::INTERRUPT_DISABLE);
        self.bus.tick(2);
        self.pc = self.read_16(0xfffa);
    }
}
