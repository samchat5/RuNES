use nes::ines_parser::File;
use nes::sdl::SDLApp;

fn main() {
    // Game ROMS -----------------------------------------------------------------------------------
    // let rom = File::new("roms/excitebike.nes");
    // let rom = File::new("roms/zelda.nes");
    let rom = File::new("roms/mario.nes");
    // let rom = File::new("roms/pacman.nes");

    let mut sdl = SDLApp::new(rom);
    sdl.run();
}
