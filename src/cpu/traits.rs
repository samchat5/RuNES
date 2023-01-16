use super::cpu::{AddressingMode, Register, Status};

pub trait LoadStore {
    fn ld(&mut self, mode: AddressingMode, reg: Register);

    fn st(&mut self, mode: AddressingMode, reg: Register);

    fn lda(&mut self, mode: AddressingMode) {
        self.ld(mode, Register::A);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        self.ld(mode, Register::X);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        self.ld(mode, Register::Y);
    }

    fn sta(&mut self, mode: AddressingMode) {
        self.st(mode, Register::A);
    }

    fn stx(&mut self, mode: AddressingMode) {
        self.st(mode, Register::X);
    }

    fn sty(&mut self, mode: AddressingMode) {
        self.st(mode, Register::Y);
    }
}

pub trait RegisterTransfer {
    fn transfer(&mut self, from: Register, to: Register);

    fn tax(&mut self) {
        self.transfer(Register::A, Register::X);
    }

    fn tay(&mut self) {
        self.transfer(Register::A, Register::Y);
    }

    fn txa(&mut self) {
        self.transfer(Register::X, Register::A);
    }

    fn tya(&mut self) {
        self.transfer(Register::Y, Register::A);
    }
}

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

pub enum LogicalOp {
    EOR,
    AND,
    ORA,
}

pub trait Logical {
    fn bit_op(&mut self, mode: AddressingMode, op: LogicalOp);

    fn bit(&mut self, mode: AddressingMode);

    fn and(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::AND);
    }

    fn ora(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::ORA);
    }

    fn eor(&mut self, mode: AddressingMode) {
        self.bit_op(mode, LogicalOp::EOR);
    }
}

pub enum ArithOp {
    ADC,
    SBC,
}

pub trait Arithmetic {
    fn arith(&mut self, mode: AddressingMode, op: ArithOp);

    fn cmpr(&mut self, mode: AddressingMode, reg: Register);

    fn adc(&mut self, mode: AddressingMode) {
        self.arith(mode, ArithOp::ADC);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        self.arith(mode, ArithOp::SBC);
    }

    fn cmp(&mut self, mode: AddressingMode) {
        self.cmpr(mode, Register::A);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        self.cmpr(mode, Register::X);
    }

    fn cpy(&mut self, mode: AddressingMode) {
        self.cmpr(mode, Register::Y);
    }
}
pub enum IncDec {
    INC,
    DEC,
}

pub trait IncDecOps {
    fn inc_dec(&mut self, mode: AddressingMode, op: IncDec);

    fn inc_dec_reg(&mut self, reg: Register, op: IncDec);

    fn inc(&mut self, mode: AddressingMode) {
        self.inc_dec(mode, IncDec::INC);
    }

    fn inx(&mut self) {
        self.inc_dec_reg(Register::X, IncDec::INC);
    }

    fn iny(&mut self) {
        self.inc_dec_reg(Register::Y, IncDec::INC);
    }

    fn dec(&mut self, mode: AddressingMode) {
        self.inc_dec(mode, IncDec::DEC);
    }

    fn dex(&mut self) {
        self.inc_dec_reg(Register::X, IncDec::DEC);
    }

    fn dey(&mut self) {
        self.inc_dec_reg(Register::Y, IncDec::DEC);
    }
}

pub enum ShiftOp {
    ASL,
    LSR,
}

pub trait Shift {
    fn shift(&mut self, mode: AddressingMode, op: ShiftOp);

    fn rol(&mut self, mode: AddressingMode);

    fn ror(&mut self, mode: AddressingMode);

    fn asl(&mut self, mode: AddressingMode) {
        self.shift(mode, ShiftOp::ASL);
    }

    fn lsr(&mut self, mode: AddressingMode) {
        self.shift(mode, ShiftOp::LSR);
    }
}

pub trait Jumps {
    fn jmp(&mut self, mode: AddressingMode);

    fn jsr(&mut self);

    fn rts(&mut self);
}

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

pub trait SysFuncs {
    fn nop(&mut self) {}

    fn brk(&mut self);

    fn rti(&mut self);
}
