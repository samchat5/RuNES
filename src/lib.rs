extern crate core;

#[warn(clippy::all)]
#[deny(clippy::correctness)]
#[warn(clippy::suspicious)]
#[warn(clippy::style)]
#[warn(clippy::complexity)]
#[warn(clippy::perf)]
#[warn(clippy::pedantic)]
#[warn(clippy::cargo)]
pub mod apu;
pub mod config;
pub mod bus;
pub mod cpu;
pub mod frame;
pub mod ines_parser;
pub mod joypad;
pub mod mappers;
pub mod ppu;
pub mod sdl;
