use crate::core::cpu::{Status, CPU};

pub trait SysFuncs {
    fn nop(&mut self);

    fn brk(&mut self);

    fn rti(&mut self);

    fn irq(&mut self);
}

impl SysFuncs for CPU<'_> {
    fn nop(&mut self) {
        self.get_operand_val();
    }

    fn brk(&mut self) {
        self.push_word(self.pc + 1);
        let flags = self.status.bits() | Status::BREAK.bits() | Status::BREAK2.bits();
        if self.need_nmi {
            self.need_nmi = false;
            self.push(flags);
            self.status.set(Status::INTERRUPT_DISABLE, true);
            self.pc = self.memory_read_word(0xfffa);
        } else {
            self.push(flags);
            self.status.set(Status::INTERRUPT_DISABLE, true);
            self.pc = self.memory_read_word(0xfffe);
        }
        self.prev_need_nmi = false;
    }

    fn rti(&mut self) {
        self.dummy_read();
        self.status = Status::from_bits(self.pop()).unwrap();
        self.pc = self.pop_word();
    }

    fn irq(&mut self) {
        self.dummy_read();
        self.dummy_read();
        self.push_word(self.pc);
        if self.need_nmi {
            self.need_nmi = false;
            self.push(self.status.bits() | Status::BREAK2.bits());
            self.status.set(Status::INTERRUPT_DISABLE, true);
            self.pc = self.memory_read_word(0xfffa);
        } else {
            self.push(self.status.bits() | Status::BREAK2.bits());
            self.status.set(Status::INTERRUPT_DISABLE, true);
            self.pc = self.memory_read_word(0xfffe);
        }
    }
}
