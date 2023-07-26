#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    FourStep = 0,
    FiveStep = 1,
}

pub enum IRQSignal {
    Clear,
    Set,
    None,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum FrameType {
    None,
    QuarterFrame,
    HalfFrame,
}

const STEP_CYCLES: [[u16; 6]; 2] = [
    [7457, 14913, 22371, 29828, 29829, 29830],
    [7457, 14913, 22371, 29829, 37281, 37282],
];
const FRAME_TYPES: [[FrameType; 6]; 2] = [
    [
        FrameType::QuarterFrame,
        FrameType::HalfFrame,
        FrameType::QuarterFrame,
        FrameType::None,
        FrameType::HalfFrame,
        FrameType::None,
    ],
    [
        FrameType::QuarterFrame,
        FrameType::HalfFrame,
        FrameType::QuarterFrame,
        FrameType::None,
        FrameType::HalfFrame,
        FrameType::None,
    ],
];

pub struct FrameCounter {
    previous_cycle: i32,
    pub step: usize,
    pub mode: Mode,
    write_delay: i8,
    inhibit_irq: bool,

    block_tick: u8,
    new_val: i16,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            previous_cycle: 0 as i32,
            step: 0,
            mode: Mode::FourStep,
            write_delay: 3,
            inhibit_irq: false,
            block_tick: 0,
            new_val: 0x00,
        }
    }
}

impl FrameCounter {
    pub fn clock(&mut self, cycles_to_run: &mut i32, mut callback: impl FnMut(FrameType)) -> (IRQSignal, u32) {
        let mut cycles_ran = 0u32;
        let mut signal = IRQSignal::None;

        if self.previous_cycle + *cycles_to_run
            >= STEP_CYCLES[self.mode as usize][self.step] as i32
        {
            if !self.inhibit_irq && self.mode == Mode::FourStep && self.step >= 3 {
                signal = IRQSignal::Set;
            }

            let typ = FRAME_TYPES[self.mode as usize][self.step];
            if typ != FrameType::None && self.block_tick == 0 {
                (callback)(typ);
                self.block_tick = 2;
            }

            if (STEP_CYCLES[self.mode as usize][self.step] as i32) < self.previous_cycle {
                cycles_ran = 0
            } else {
                cycles_ran =
                    (STEP_CYCLES[self.mode as usize][self.step] as i32 - self.previous_cycle).abs() as u32;
            }

            *cycles_to_run -= cycles_ran as i32;

            self.step += 1;
            if self.step == 6 {
                self.step = 0;
                self.previous_cycle = 0;
            } else {
                self.previous_cycle += cycles_ran as i32;
            }
        } else {
            cycles_ran = *cycles_to_run as u32;
            *cycles_to_run = 0;
            self.previous_cycle += cycles_ran as i32;
        }

        if self.new_val >= 0 {
            self.write_delay -= 1;
            if self.write_delay == 0 {
                self.mode = if self.new_val & 0x80 != 0 {
                    Mode::FiveStep
                } else {
                    Mode::FourStep
                };

                self.write_delay = -1;
                self.step = 0;
                self.previous_cycle = 0;
                self.new_val = -1;

                if self.mode == Mode::FiveStep && self.block_tick == 0 {
                    (callback)(FrameType::HalfFrame); 
                    self.block_tick = 2;
                }
            }
        }

        if self.block_tick > 0 {
            self.block_tick -= 1;
        }

        (signal, cycles_ran)
    }

    pub fn write(&mut self, val: u8, cycle: usize) -> IRQSignal {
        self.new_val = val as i16;

        if cycle & 1 == 1 {
            self.write_delay = 4;
        } else {
            self.write_delay = 3;
        }

        self.inhibit_irq = (val & 0x40) == 0x40;
        if self.inhibit_irq {
            IRQSignal::Clear
        } else {
            IRQSignal::None
        }
    }

    pub fn need_to_run(&self, cycles_to_run: u32) -> bool {
        return self.new_val >= 0
            || self.block_tick > 0
            || (self.previous_cycle + cycles_to_run as i32)
                >= (STEP_CYCLES[self.mode as usize][self.step] as i32) - 1;
    }
}
