pub mod cpu;
pub mod ines_parser;
use cpu::cpu::CPU;
use ines_parser::File;

fn main() {
    let file = File::new("tests/nestest.nes");
    let mut cpu = CPU::new();
    cpu.load_prg_rom(file);
    cpu.run();
}
