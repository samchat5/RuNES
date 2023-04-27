use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use std::collections::HashMap;

use nes::joypad::{Buttons, Joypad};
use nes::ppu::PPU;
use nes::{bus::Bus, cpu::CPU, ines_parser::File};

fn main() {
    // Load the game
    // let rom = File::new("tests/instr_test-v5/official_only.nes"); // Mapper 1

    // let rom = File::new("tests/branch_timing_tests/1.Branch_Basics.nes"); // Passes
    // let rom = File::new("tests/branch_timing_tests/2.Backward_Branch.nes"); // Passes
    // let rom = File::new("tests/branch_timing_tests/3.Forward_Branch.nes"); // Passes

    // let rom = File::new("tests/blargg_ppu_tests_2005.09.15b/palette_ram.nes"); // Passes
    // let rom = File::new("tests/blargg_ppu_tests_2005.09.15b/power_up_palette.nes"); // --fix
    // let rom = File::new("tests/blargg_ppu_tests_2005.09.15b/sprite_ram.nes"); // -- fix
    // let rom = File::new("tests/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes"); // -- fix
    // let rom = File::new("tests/blargg_ppu_tests_2005.09.15b/vram_access.nes"); // -- fix

    // let rom = File::new("tests/nestest/nestest.nes");

    // let rom = File::new("roms/mario.nes");
    // let rom = File::new("roms/pacman.nes");
    let rom = File::new("roms/excitebike.nes");

    create(rom);
}

fn create(rom: File) {
    let scale_x = 3f32;
    let scale_y = 3f32;

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::W, Buttons::UP);
    key_map.insert(Keycode::S, Buttons::DOWN);
    key_map.insert(Keycode::D, Buttons::RIGHT);
    key_map.insert(Keycode::A, Buttons::LEFT);
    key_map.insert(Keycode::U, Buttons::SELECT);
    key_map.insert(Keycode::I, Buttons::START);
    key_map.insert(Keycode::K, Buttons::A);
    key_map.insert(Keycode::J, Buttons::B);

    // Init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", (256.0 * scale_x) as u32, (240.0 * scale_y) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(scale_x, scale_y).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut timer = sdl_context.timer().unwrap();
    let bus = Bus::new(rom, move |ppu: &mut PPU, joypad: &mut Joypad| {
        let start_ticks = timer.ticks();
        let frame = ppu.curr_frame;
        texture.update(None, &(frame).image, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(button) = key_map.get(&keycode.unwrap()) {
                        println!("{:?} ", button);
                        joypad.buttons.set(*button, true)
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(button) = key_map.get(&keycode.unwrap()) {
                        joypad.buttons.set(*button, false)
                    }
                }
                _ => (),
            }
        }
        let end_ticks = timer.ticks();
        if end_ticks - start_ticks < 16 {
            timer.delay(16 - (end_ticks - start_ticks));
        }
    });

    let mut cpu = CPU::new(bus);
    cpu.set_sink(Box::new(
        std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open("log.log")
            .unwrap(),
    ));
    cpu.reset();
    cpu.run(u64::MAX);
}
