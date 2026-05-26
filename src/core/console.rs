use std::{
    io::Write,
    path::PathBuf,
};
use crate::{config::Config, ines_parser::NESFile};
use super::{bus::Bus, cpu::CPU, frame::Frame, joypad::Buttons};

pub struct Console {
    pub cpu: CPU,
    pub rom_hash: u64,
}

impl Console {
    pub fn new(rom: NESFile) -> Self {
        let mut cpu = CPU::new(Bus::new(&rom));

        if Config::get_bool("enable_logging", false) {
            cpu.set_sink(Box::new(
                std::fs::File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(Config::get_string_with_default("logging_path", "log.log"))
                    .unwrap(),
            ));
            cpu.enable_logging();
        }
        cpu.reset();

        Console {
            cpu,
            rom_hash: rom.hash,
        }
    }

    pub fn run_frame(&mut self) -> Vec<i16> {
        self.cpu.run_until_frame();
        let mut samples = Vec::with_capacity(1024);
        self.cpu.bus.apu.output_buffer.end_frame(&mut samples);
        samples
    }

    pub fn set_joypad(&mut self, button: Buttons, pressed: bool) {
        self.cpu.bus.joypad.buttons.set(button, pressed);
    }

    pub fn frame(&self) -> &Frame {
        &self.cpu.bus.ppu.curr_frame
    }

    pub fn dump_save_to_path(&self, file: PathBuf) -> std::io::Result<()> {
        let mapper = self.cpu.bus.mapper.lock().unwrap();
        let save = mapper.dump_save();
        std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file)?
            .write_all(save)?;
        Ok(())
    }

    pub fn dump_save(&self) -> std::io::Result<()> {
        if let Some(save_dir_str) = Config::get_string("save_directory") {
            let mut save_path = PathBuf::from(save_dir_str);
            save_path.push(format!("{}.sav", self.rom_hash));
            println!("{}", save_path.to_str().unwrap());
            self.dump_save_to_path(save_path)?;
        }
        Ok(())
    }

    pub fn load_save(&self, file: PathBuf) -> std::io::Result<()> {
        let save = std::fs::read(file)?;
        let mut mapper = self.cpu.bus.mapper.lock().unwrap();
        mapper.load_save(save.as_slice());
        Ok(())
    }
}
