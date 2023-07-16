use nes::ines_parser::File;
use nes::sdl::SDLApp;

fn main() {
    // Game ROMS -----------------------------------------------------------------------------------
    // let rom = File::new("roms/excitebike.nes");
    let rom = File::new("roms/zelda.nes");

    // Pass
    // let rom = File::new("tests/blargg_apu_2005.07.30/01.len_ctr.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/02.len_table.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/09.reset_timing.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/11.len_reload_timing.nes");

    // Fail
    // let rom = File::new("tests/blargg_apu_2005.07.30/04.clock_jitter.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/05.len_timing_mode0.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/06.len_timing_mode1.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/07.irq_flag_timing.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/08.irq_timing.nes");
    // let rom = File::new("tests/blargg_apu_2005.07.30/10.len_halt_timing.nes");

    // let rom = File::new("roms/mario.nes");
    // let rom = File::new("roms/pacman.nes");

    let mut sdl = SDLApp::new(rom);
    sdl.run();
}
