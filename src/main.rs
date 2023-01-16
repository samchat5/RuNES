pub mod cpu;
pub mod ines_parser;
use cpu::cpu::CPU;
use ines_parser::File;

fn main() {
    let file = File::new("tests/nestest/nestest.nes");
    let mut cpu = CPU::new();
    cpu.load_prg_rom(file);
    cpu.reset_with_val(0xc000);
    cpu.run();
}
