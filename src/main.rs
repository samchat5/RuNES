use nes::{frontend::sdl::SDLApp, ines_parser::File};

fn main() {
    // Game ROMS -----------------------------------------------------------------------------------
    // let rom = File::new("roms/excitebike.nes");
    // let rom = File::new("roms/zelda.nes");
    let rom = File::new("roms/mario.nes");
    // let rom = File::new("roms/pacman.nes");

    // let native_options = eframe::NativeOptions::default();
    // let _ = eframe::run_native(
    //     "NES",
    //     native_options,
    //     Box::new(|_cc| Box::new(EGuiApp::new(rom))),
    // );

    let mut sdl = SDLApp::new(rom);
    sdl.run();
}
