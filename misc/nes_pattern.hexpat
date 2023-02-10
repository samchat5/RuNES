#include <std/math.pat>
#include <std/io.pat>

bitfield Flags1{
    name_table_mirror: 1;
    battery: 1;
    trainer: 1;
    four_screen_mode: 1;
    mapper_num: 4;
};

bitfield Flags2{
    console_type: 2;
    magic: 2;
    mapper_num: 4;
};

bitfield MapperMSB{
    mapper_num: 4;
    submapper_num: 4;
};

bitfield ROMSizeMSB{
    prg: 4;
    chr: 4;
};

bitfield PRGRAMEEPROMSize{
    prg_ram_shift_cnt: 4;
    eeprom_shift_cnt: 4;
};

bitfield CHRRAMSize{
    chr_ram_size: 4;
    chr_nvram_size: 4;
};

bitfield Timing{
    mode: 2;
    _unused: 6;
};

bitfield VsSystemType{
    ppu_type: 4;
    hardware_type: 4;
};

bitfield ExtendedConsoleType{
    type: 4;
    _unused: 4;
};

bitfield MiscROMs{
    misc_rom_num: 2;
    _unused: 6;
};

bitfield DefaultExpansionDevice{
    device: 6;
    _unused: 2;
};

struct Header {
    char magic[4];
    u8 prg_rom_size_lsb;
    u8 chr_rom_size_lsb;
    Flags1 flags1;
    Flags2 flags2;
    MapperMSB mapper_msb;
    ROMSizeMSB rom_size_msb;
    PRGRAMEEPROMSize prg_ram_eeprom_size;
    CHRRAMSize chr_ram_size;
    Timing timing;
    if (flags2.console_type == 1) {
        VsSystemType type;
    } else if (flags2.console_type == 3) {
        ExtendedConsoleType type;
    } else {
        u8 _unused;
    }
    MiscROMs misc_roms;
    DefaultExpansionDevice default_expansion_device;
};

fn get_prg_rom_size(Header header) {
    if (header.rom_size_msb.prg == 0xF) {
        return std::math::pow(2, header.prg_rom_size_lsb & 0xFC)
            * ((header.prg_rom_size_lsb & 0x03) * 2 + 1);
    } else {
        return (u16(header.prg_rom_size_lsb) |
            u16(header.rom_size_msb.prg) << 8) * 16384;
    }
};

fn get_chr_rom_area(Header header) {
    if (header.rom_size_msb.chr == 0xF) {
        return std::math::pow(2, header.chr_rom_size_lsb & 0xFC)
            * ((header.prg_rom_size_lsb & 0x03) * 2 + 1);
    } else {
        return (u16(header.chr_rom_size_lsb) |
            u16(header.rom_size_msb.chr) << 8) * 8192;
    }
};

struct File {
    // Header
    Header header;

    // Trainer Area
    if (header.flags1.trainer) {
        u8 trainer[512];
    }

    // PRG-ROM Area
    u8 prg_rom_area[get_prg_rom_size(header)];

    // CHR-ROM Area
    if (sizeof($) - $ != 0) {
        u8 chr_rom_area[get_chr_rom_area(header)];
    }

    // Misc ROM Area
    if (sizeof($) - $ != 0) {
        u8 misc_rom_area[sizeof($) - $];
    }
};

File file
@ 0x0;

