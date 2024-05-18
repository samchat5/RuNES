use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, StreamConfig, StreamError,
};
use crossbeam::channel::Receiver;

use crate::{config::Config, frontend::egui::ConsoleMsg, ines_parser::File};

use super::{apu, bus::Bus, cpu::CPU};

pub struct Console<'a> {
    pub cpu: CPU<'a>,
}

impl Console<'_> {
    pub fn new(rom: File) -> Self {
        let mut cpu = CPU::new(Bus::new(&rom));

        if Config::get_bool("enable_logging", false) {
            cpu.set_sink(Box::new(
                std::fs::File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(Config::get_string("logging_path", "log.log"))
                    .unwrap(),
            ));
            cpu.enable_logging();
        }
        cpu.reset();

        Console { cpu }
    }

    pub fn dump_save(&self, file: PathBuf) -> std::io::Result<()> {
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

    pub fn load_save(&self, file: PathBuf) -> std::io::Result<()> {
        let save = std::fs::read(file)?;
        let mut mapper = self.cpu.bus.mapper.lock().unwrap();
        mapper.load_save(save.as_slice());
        Ok(())
    }

    pub fn run_thread(console: Arc<Mutex<Console<'static>>>, recv: Receiver<ConsoleMsg>) {
        let (audio_send, audio_recv) = crossbeam::channel::bounded::<i16>(2048);
        let (stream, sample_rate) = Console::setup_audio(audio_recv);
        let mut samples = Vec::with_capacity(16);

        {
            let mut console = console.lock().unwrap();
            console
                .cpu
                .bus
                .apu
                .output_buffer
                .set_rates(apu::APU::CLOCK_RATE, sample_rate.0 as f64);
        }

        loop {
            for msg in recv.try_iter() {
                let mut console = console.lock().unwrap();
                match msg {
                    ConsoleMsg::JoypadDown(button) => {
                        console.cpu.bus.joypad.buttons.set(button, true)
                    }
                    ConsoleMsg::JoypadUp(button) => {
                        console.cpu.bus.joypad.buttons.set(button, false)
                    }
                    ConsoleMsg::RunFrame => {
                        // Exectue
                        console.cpu.run_until_frame();

                        // Audio
                        console.cpu.bus.apu.output_buffer.end_frame(&mut samples);
                        for sample in samples.iter() {
                            audio_send.try_send(*sample).ok();
                        }
                        stream.play().unwrap();
                        samples.clear();
                    }
                }
            }
        }
    }

    pub fn setup_audio(audio_recv: Receiver<i16>) -> (cpal::Stream, SampleRate) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let default_config: StreamConfig = device.default_output_config().unwrap().into();
        let sample_rate = default_config.sample_rate;
        let channels = default_config.channels;

        let stream = device
            .build_output_stream(
                &default_config,
                move |buf: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    Console::stream_callback(buf, audio_recv.clone(), channels);
                },
                |e: StreamError| {
                    dbg!(e);
                },
                None,
            )
            .unwrap();
        (stream, sample_rate)
    }

    fn stream_callback(buf: &mut [i16], audio_recv: Receiver<i16>, channels: u16) {
        let requested = buf.len();
        let sample_iter = audio_recv.try_iter().take(requested / channels as usize);

        let mut i = 0;
        for sample in sample_iter {
            buf[i..(i + channels as usize)].fill(sample);
            i += channels as usize;
        }
    }
}
