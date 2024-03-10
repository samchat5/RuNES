use crate::config::Config;
use crate::core::apu;
use crate::core::bus::Bus;
use crate::core::cpu::CPU;
use crate::core::joypad::Buttons;
use crate::ines_parser::File;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig, StreamError, SupportedStreamConfig};
use crossbeam::channel::{Receiver, Sender};
use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::libc::wait;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::EventPump;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, Instant};

struct TextureManager<'a> {
    texture: Texture<'a>,
}

impl<'a> TextureManager<'a> {
    fn new(creator: &'a TextureCreator<WindowContext>) -> Self {
        let texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();
        Self { texture }
    }
}

lazy_static! {
    static ref KEY_MAP: HashMap<Keycode, Buttons> = [
        (Keycode::W, Buttons::UP),
        (Keycode::S, Buttons::DOWN),
        (Keycode::D, Buttons::RIGHT),
        (Keycode::A, Buttons::LEFT),
        (Keycode::U, Buttons::SELECT),
        (Keycode::I, Buttons::START),
        (Keycode::K, Buttons::A),
        (Keycode::J, Buttons::B),
    ]
    .iter()
    .fold(HashMap::new(), |mut acc, (key, button)| {
        acc.insert(*key, *button);
        acc
    });
}

const FRAME_DURATION: Duration = Duration::from_nanos(16_666_667);

pub struct SDLApp {
    canvas: WindowCanvas,
    event_pump: EventPump,
    cpu: CPU<'static>,
    samples: Vec<i16>,
    audio_send: Sender<i16>,
    stream: cpal::Stream,
}

impl SDLApp {
    pub fn new(rom: File) -> Self {
        let scale_x = 3f32;
        let scale_y = 3f32;
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("NES", (256.0 * scale_x) as u32, (240.0 * scale_y) as u32)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.set_scale(scale_x, scale_y).unwrap();

        let event_pump = sdl_context.event_pump().unwrap();
        let mut cpu = CPU::new(Bus::new(&rom));

        let (audio_send, audio_recv) = crossbeam::channel::bounded::<i16>(2048);
        let (stream, sample_rate) = Self::setup_audio(audio_recv);

        cpu.bus
            .apu
            .output_buffer
            .set_rates(apu::APU::CLOCK_RATE, sample_rate.0 as f64);

        Self {
            cpu,
            canvas,
            event_pump,
            samples: Vec::with_capacity(16),
            audio_send,
            stream,
        }
    }

    fn handle_keyevent(&mut self) {
        let joypad = &mut self.cpu.bus.joypad;
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(button) = KEY_MAP.get(&keycode.unwrap()) {
                        println!("{:?} ", button);
                        joypad.buttons.set(*button, true)
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(button) = KEY_MAP.get(&keycode.unwrap()) {
                        joypad.buttons.set(*button, false)
                    }
                }
                _ => (),
            }
        }
    }

    pub fn run(&mut self) {
        if Config::get_bool("enable_logging", false) {
            self.cpu.set_sink(Box::new(
                std::fs::File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(Config::get_string("logging_path", "log.log"))
                    .unwrap(),
            ));
            self.cpu.enable_logging();
        }
        self.cpu.reset();
        self.game_loop();
    }

    fn game_loop(&mut self) {
        let texture_creator = self.canvas.texture_creator();
        let mut texture_manager = TextureManager::new(&texture_creator);
        let texture = &mut texture_manager.texture;

        loop {
            self.handle_keyevent();

            let deadline = Instant::now() + FRAME_DURATION;
            self.execute(texture);
            let now = Instant::now();
            if now > deadline {
                println!("MISSED DEADLINE");
            } else {
                let delta = deadline.duration_since(now);
                sleep(delta);
            }
        }
    }

    fn execute(&mut self, texture: &mut Texture) {
        // Run CPU
        self.cpu.run_until_frame();

        // Audio
        self.cpu.bus.apu.output_buffer.end_frame(&mut self.samples);
        for sample in self.samples.iter() {
            self.audio_send.try_send(*sample).ok();
        }
        self.stream.play().unwrap();
        self.samples.clear();

        // Render
        texture
            .update(None, &self.cpu.bus.ppu.curr_frame.image, 256 * 3)
            .unwrap();
        self.canvas.copy(texture, None, None).unwrap();
        self.canvas.present();
    }

    fn setup_audio(audio_recv: Receiver<i16>) -> (cpal::Stream, SampleRate) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let default_config: StreamConfig = device.default_output_config().unwrap().into();
        let sample_rate = default_config.sample_rate;
        let channels = default_config.channels;

        let stream = device
            .build_output_stream(
                &default_config,
                move |buf: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    SDLApp::stream_callback(buf, audio_recv.clone(), channels);
                },
                SDLApp::stream_err,
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

    fn stream_err(e: StreamError) {
        dbg!(e);
    }
}
