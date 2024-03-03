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
pub mod config;
pub mod core;
pub mod frontend;
pub mod ines_parser;
