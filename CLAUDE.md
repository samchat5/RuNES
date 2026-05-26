# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Run/Test

```bash
cargo run --release          # Run the emulator
cargo test                   # Run all integration tests
cargo clippy                 # Lint (strict: all, pedantic, nursery, cargo, complexity, perf, style, suspicious)
```

Tests live in `tests/test.rs` as a single file using an `integration_tests!` macro. Each test runs a ROM for N frames then asserts the frame buffer hash matches a known value. There are no unit tests — everything is frame-hash integration tests.

## Architecture

A NES emulator in Rust. The entry point is `src/main.rs`, which launches an `eframe` (egui) window via `EGuiApp`. The emulator core runs on a separate thread (`Console::run_thread`), communicating with the UI through a `crossbeam` channel (`ConsoleMsg` enum).

### Core hierarchy (`src/core/`)

- **`Console`** — Owns a `CPU`, manages save/load of SRAM to `./saves/{rom_hash}.sav`, and runs the main emulation loop on a dedicated thread. Each frame: `cpu.run_until_frame()` → read APU samples → push to `cpal` audio output.
- **`CPU`** — 6502 CPU emulation. Instruction dispatch is in `op.rs` (a table of `OPS`), with each instruction group in `cpu_units/` (arithmetic, branches, jumps, load_store, etc.). Includes an optional instruction tracer (`tracer.rs`) gated by the `enable_logging` config key, which writes to the path in `logging_path`.
- **`Bus`** — Central address-space router. Maps addresses to CPU RAM (`$0000–$1FFF`), PPU registers (`$2000–$3FFF`), APU/IO registers (`$4000–$401F`), and cart address space via the mapper. All reads/writes go through the bus.
- **`PPU`** — 2C02 PPU emulation. Renders 256×240 frames into a `Frame` struct (flat `[u8; 256*240*3]` RGB buffer). Register state in `ppu/registers/` (control, mask, status, scroll). Tracks cycle/scanline for timing. `set_pixel` marks zero pixels in `is_zero` for sprite-0 hit detection.
- **`APU`** — Audio processing unit with Pulse (2), Triangle, Noise, and DMC channels. Uses `BlipBuf` for band-limited synthesis to resample from NES clock rate to host sample rate. Frame counter (`frame_counter.rs`) drives length counters and envelope units.
- **`Mapper` trait** (`mappers/mod.rs`) — Cartridge memory mapping. Implementations: `NROM` (0), `MMC1` (1), `CNROM` (3). Factory created via `MapperFactory::from_file()` producing a `SharedMapper` (`Arc<Mutex<Box<dyn Mapper + Send>>>`).
- **`Joypad`** — Standard NES controller. Button state is `Buttons` bitflags, key mappings hardcoded in `frontend/egui.rs` (WASD for D-pad, J/K for B/A, U/I for Select/Start).

### Frontend (`src/frontend/`)

- **`egui.rs`** — The `EGuiApp` struct implements `eframe::App`. On each frame it sends `RunFrame` to the console thread, then displays the resulting `Frame` as a texture. Handles ROM loading/saving via `rfd` file dialogs and keyboard input via egui key events.
- **`blip_buf.rs`** — Port of Blip_Buffer band-limited synthesis library for audio resampling.

### ROM parsing (`src/ines_parser.rs`)

Parses NES 2.0 `.nes` files. Returns an `NESFile` struct with PRG/CHR ROM data and header flags.

### Configuration (`src/config.rs`, `config.toml`)

Uses the `config` crate to read `config.toml`. Keys: `enable_logging` (CPU instruction trace), `logging_path`, `save_directory`.

## Key design notes

- The console thread loop blocks on `recv.iter()` and processes messages sequentially — UI frame requests, joypad changes, etc. This means the UI and emulation are decoupled but synchronized per-frame.
- PPU open bus behavior is emulated: writes update `open_bus`, reads apply bitmask-based open bus values.
- Save files are automatically loaded on startup if present in `save_directory`, matched by ROM hash.
- The `#[warn(clippy::all, clippy::pedantic, clippy::nursery, ...)]` in `lib.rs` means strict linting is enforced project-wide.
