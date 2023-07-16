#[derive(PartialEq, Eq, Copy, Clone)]
pub enum FrameCounterMode {
    FourStep = 0,
    FiveStep = 1,
}

const STEP_CYCLES: [[u16; 6]; 2] = [
    [7457, 7456, 7458, 7457, 1, 1],
    [7457, 7456, 7458, 7458, 7452, 1],
];

pub struct FrameCounter {
    cycles: u16,
    pub step: usize,
    pub mode: FrameCounterMode,
    write_buffer: Option<u8>,
    write_delay: u8,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            cycles: STEP_CYCLES[0][0],
            step: 0,
            mode: FrameCounterMode::FourStep,
            write_buffer: None,
            write_delay: 0,
        }
    }
}

impl FrameCounter {
    pub fn update(&mut self) -> bool {
        if let Some(val) = self.write_buffer {
            self.write_delay -= 1;
            if self.write_delay == 0 {
                self.reload(val);
                self.write_buffer = None;
                return true;
            }
        }
        false
    }

    pub fn reload(&mut self, val: u8) {
        self.mode = if val & 0x80 == 0x80 {
            FrameCounterMode::FiveStep
        } else {
            FrameCounterMode::FourStep
        };
        self.step = 0;
        self.cycles = STEP_CYCLES[self.mode as usize][self.step];

        if self.mode == FrameCounterMode::FourStep {
            self.clock();
        }
    }

    pub fn clock(&mut self) -> usize {
        if self.cycles > 0 {
            self.cycles -= 1;
        }
        if self.cycles == 0 {
            let clock = self.step;
            self.step += 1;
            if self.step > 5 {
                self.step = 0;
            }
            self.cycles = STEP_CYCLES[self.mode as usize][self.step];
            clock
        } else {
            0
        }
    }

    pub fn write(&mut self, val: u8, cycle: usize) {
        self.write_buffer = Some(val);
        self.write_delay = if cycle & 1 == 1 { 4 } else { 3 }
    }
}
