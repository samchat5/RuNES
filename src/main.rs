use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

use nes::{bus::Bus, cpu::CPU, frame::Frame, ines_parser::File, ppu::PPU};

fn main() {
    let scale_x = 1f32;
    let scale_y = 1f32;

    // Init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Tile viewer",
            (256.0 * scale_x) as u32,
            (240.0 * scale_y) as u32,
        )
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

    // Load the game
    let rom = File::new("roms/pacman.nes");

    let bus = Bus::new(rom, move |ppu: &PPU| {
        let frame = ppu.render();
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
                _ => (),
            }
        }
    });

    let mut cpu = CPU::new(bus);

    cpu.reset();
    cpu.run(u64::MAX);
}
