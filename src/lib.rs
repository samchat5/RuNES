extern crate core;

#[warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::suspicious
)]
#[deny(clippy::correctness)]
#[allow(clippy::cast_precision_loss)]
pub mod apu;
pub mod bus;
pub mod config;
pub mod cpu;
pub mod frame;
pub mod ines_parser;
pub mod joypad;
pub mod mappers;
pub mod ppu;
pub mod sdl;
