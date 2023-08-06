use crate::bus::Bus;
use crate::config::Config;
use crate::cpu::CPU;
use crate::ines_parser::File;
use crate::joypad::Buttons;
use lazy_static::lazy_static;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::EventPump;
use std::collections::HashMap;
use std::time::SystemTime;

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

pub struct SDLApp {
    canvas: WindowCanvas,
    event_pump: EventPump,
    queue: AudioQueue<f32>,
    cpu: CPU<'static>,
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

        let audio = sdl_context.audio().unwrap();
        let queue = audio
            .open_queue(
                None,
                &AudioSpecDesired {
                    freq: Some(44_100),
                    channels: Some(1),
                    samples: Some(4096),
                },
            )
            .unwrap();
        queue.resume();

        let cpu = CPU::new(Bus::new(&rom));

        Self {
            cpu,
            canvas,
            event_pump,
            queue,
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
        let mut frame_start = SystemTime::now();

        loop {
            self.handle_keyevent();

            let force = self.queue.size() < 44100 / 2;
            if force || frame_start.elapsed().unwrap() > std::time::Duration::from_micros(16666) {
                self.execute(texture);
                frame_start = SystemTime::now();
            }
        }
    }

    fn emulate(&mut self, texture: &mut Texture) {
        self.cpu.run_until_frame();
        texture
            .update(None, &self.cpu.bus.ppu.curr_frame.image, 256 * 3)
            .unwrap();
    }

    fn execute(&mut self, texture: &mut Texture) {
        self.emulate(texture);
        let apu = &mut self.cpu.bus.apu;
        self.queue.queue_audio(apu.get_buffer()).unwrap();
        apu.clear_buffer();

        self.canvas.copy(texture, None, None).unwrap();

        self.canvas.present();
    }
}
