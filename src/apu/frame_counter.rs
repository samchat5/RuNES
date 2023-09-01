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
    write_buffer: Option<u8>,
    write_delay: i8,
    block_tick: u8,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            previous_cycle: 0,
            step: 0,
            mode: Mode::FourStep,
            write_delay: 3,
            block_tick: 0,
            write_buffer: None,
        }
    }
}

impl FrameCounter {
    pub fn clock(
        &mut self,
        inhibit_irq: bool,
        cycles_to_run: &mut i32,
        mut callback: impl FnMut(FrameType),
    ) -> (IRQSignal, u32) {
        let cycles_ran;
        let mut signal = IRQSignal::None;

        if self.previous_cycle + *cycles_to_run >= i32::from(STEP_CYCLES[self.mode as usize][self.step])
        {
            if !inhibit_irq && self.mode == Mode::FourStep && self.step >= 3 {
                signal = IRQSignal::Set;
            }

            let typ = FRAME_TYPES[self.mode as usize][self.step];
            if typ != FrameType::None && self.block_tick == 0 {
                (callback)(typ);
                self.block_tick = 2;
            }

            cycles_ran = if i32::from(STEP_CYCLES[self.mode as usize][self.step]) < self.previous_cycle {
                0
            } else {
                (i32::from(STEP_CYCLES[self.mode as usize][self.step]) - self.previous_cycle).unsigned_abs()
            };

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

        if let Some(wb) = self.write_buffer {
            self.write_delay -= 1;
            if self.write_delay == 0 {
                self.mode = if wb & 0x80 == 0 {
                    Mode::FourStep
                } else {
                    Mode::FiveStep
                };

                self.write_delay = -1;
                self.step = 0;
                self.previous_cycle = 0;
                self.write_buffer = None;

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

    #[must_use] pub fn need_to_run(&self, cycles_to_run: u32) -> bool {
        self.write_buffer.is_some()
            || self.block_tick > 0
            || (self.previous_cycle + cycles_to_run as i32)
                >= i32::from(STEP_CYCLES[self.mode as usize][self.step]) - 1
    }

    pub fn write(&mut self, val: u8, cycle: usize) {
        self.write_buffer = Some(val);
        self.write_delay = if cycle & 1 == 1 { 4 } else { 3 }
    }
}
