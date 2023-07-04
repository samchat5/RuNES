use crate::bus::Bus;
use crate::config::CONFIG;
use crate::cpu::CPU;
use crate::ines_parser::File;
use crate::joypad::{Buttons, Joypad};
use crate::ppu::PPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::{EventPump, TimerSubsystem};
use std::collections::HashMap;

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

pub struct SDLApp {
    rom: File,
    key_map: HashMap<Keycode, Buttons>,
    timer: TimerSubsystem,
    canvas: WindowCanvas,
    event_pump: EventPump,
    texture_creator: TextureCreator<WindowContext>,
}

impl SDLApp {
    pub fn new(rom: File) -> Self {
        let mut key_map = HashMap::new();
        key_map.insert(Keycode::W, Buttons::UP);
        key_map.insert(Keycode::S, Buttons::DOWN);
        key_map.insert(Keycode::D, Buttons::RIGHT);
        key_map.insert(Keycode::A, Buttons::LEFT);
        key_map.insert(Keycode::U, Buttons::SELECT);
        key_map.insert(Keycode::I, Buttons::START);
        key_map.insert(Keycode::K, Buttons::A);
        key_map.insert(Keycode::J, Buttons::B);

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
        let event_pump = sdl_context.event_pump().unwrap();
        let timer = sdl_context.timer().unwrap();
        let texture_creator = canvas.texture_creator();

        canvas.set_scale(scale_x, scale_y).unwrap();

        Self {
            rom,
            key_map,
            timer,
            canvas,
            texture_creator,
            event_pump,
        }
    }

    pub fn run(&mut self) {
        let texture_manager = TextureManager::new(&self.texture_creator);
        let mut texture = texture_manager.texture;

        let bus = Bus::new(&self.rom, |ppu: &mut PPU, joypad: &mut Joypad| {
            let start_ticks = self.timer.ticks();
            let frame = ppu.curr_frame;
            texture.update(None, &(frame).image, 256 * 3).unwrap();
            self.canvas.copy(&texture, None, None).unwrap();
            self.canvas.present();
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown { keycode, .. } => {
                        if let Some(button) = self.key_map.get(&keycode.unwrap()) {
                            println!("{:?} ", button);
                            joypad.buttons.set(*button, true)
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(button) = self.key_map.get(&keycode.unwrap()) {
                            joypad.buttons.set(*button, false)
                        }
                    }
                    _ => (),
                }
            }
            let end_ticks = self.timer.ticks();
            if end_ticks - start_ticks < 16 {
                self.timer.delay(16 - (end_ticks - start_ticks));
            }
        });

        let mut cpu = CPU::new(bus);

        if CONFIG.get_bool("enable_logging").unwrap_or(false) {
            cpu.set_sink(Box::new(
                std::fs::File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(
                        CONFIG
                            .get_string("logging_path")
                            .unwrap_or_else(|_| "log.log".to_string()),
                    )
                    .unwrap(),
            ));
            cpu.enable_logging();
        }
        cpu.reset();
        cpu.run(
            CONFIG
                .get_int("run_cycles")
                .map_or_else(|_| u64::MAX, |x| x as u64),
        );
    }
}
