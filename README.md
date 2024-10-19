# RuNES
Yet another NES Emulator in Rust :)

## Compatability
This project has only been tested on WSL (Ubuntu), and likely works on any linux distribution.

## Build + Run
Simply run `cargo run --release` in the main project, and you're running!

### Loading a Game
To run a ROM, click `Load ROM` in the toolbar and pick a NES 2.0 compatible `.nes` file. To save a savefile (anything that is stored to the NES's SRAM), click `Save File` _after_ loading a ROM. This can be reloaded explcitly with `Load File`

When saving, it is automatically stored to `./saves/` in the project directory. When reloading the app, the emulator will detect any savefiles in the directory and load them in automatically

## Controls
As of yet, these are not remappable, to keyboard nor gamepad.

`W` - `Up` 

`A` - `Left`

`S` - `Down`

`D` - `Right`

`U` - `Select`

`I` - `Start`

`J` - `B`

`K` - `A`

## Supported Mappers
- [x] 000 (NROM) - 247/2447 Games (10.1%)
- [x] 001 (MMC1) - 680/2447 Games (27.8%)
- [ ] Everything else

In total, this emulator supports **37.9%** of all NES games according to [https://nescartdb.com]()

## Features to Add
- [ ] More mappers
- [ ] Noise channel on APU
- [ ] RetroArch/libretro support
- [ ] Windows + macOS support
- [ ] Clean up that damn PPU code
