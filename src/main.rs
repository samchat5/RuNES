use nes::{cpu::CPU, ines_parser::File};
use std::{cell::RefCell, io::BufWriter};

fn main() {
    let file = File::new("tests/nestest/nestest.nes");
    let cpu = RefCell::new(CPU::new());

    // Set logging
    let log = Box::new(BufWriter::new(
        std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open("tests/nestest/log.log")
            .unwrap(),
    ));
    cpu.borrow_mut().set_sink(log);

    cpu.borrow_mut().load_prg_rom(file);
    cpu.borrow_mut().reset_with_val(0xc000);
    cpu.borrow_mut().run(26554);
}
