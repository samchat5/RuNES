#![allow(non_camel_case_types, dead_code, clippy::upper_case_acronyms)]

const NES_MAGIC: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];

enum NameTableMirrorType {
    HORIZONTAL_OR_MAPPER,
    VERTICAL,
}

impl NameTableMirrorType {
    fn get(val: u8) -> Self {
        match val {
            0 => NameTableMirrorType::HORIZONTAL_OR_MAPPER,
            1 => NameTableMirrorType::VERTICAL,
            _ => panic!("Invalid NameTableMirrorType value"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Flags1(u8);
enum Flags1Enum {
    NAME_TABLE_MIRROR,
    BATTERY,
    TRAINER,
    FOUR_SCREEN_MODE,
    MAPPER_NUM,
}
impl Flags1 {
    fn get(self, arg: Flags1Enum) -> u8 {
        match arg {
            Flags1Enum::NAME_TABLE_MIRROR => self.0 & 0x01,
            Flags1Enum::BATTERY => (self.0 & 0x02) >> 1,
            Flags1Enum::TRAINER => (self.0 & 0x04) >> 2,
            Flags1Enum::FOUR_SCREEN_MODE => (self.0 & 0x08) >> 3,
            Flags1Enum::MAPPER_NUM => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Flags2(u8);
enum Flags2Enum {
    CONSOLE_TYPE,
    MAGIC,
    MAPPER_NUM,
}
impl Flags2 {
    fn get(self, arg: Flags2Enum) -> u8 {
        match arg {
            Flags2Enum::CONSOLE_TYPE => self.0 & 0x03,
            Flags2Enum::MAGIC => (self.0 & 0x0c) >> 2,
            Flags2Enum::MAPPER_NUM => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct MapperMSB(u8);
enum MapperMSBEnum {
    MAPPER_NUM,
    SUBMAPPER_NUM,
}
impl MapperMSB {
    fn get(self, arg: MapperMSBEnum) -> u8 {
        match arg {
            MapperMSBEnum::MAPPER_NUM => self.0 & 0x0f,
            MapperMSBEnum::SUBMAPPER_NUM => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ROMSizeMSB(u8);
enum ROMSizeMSBEnum {
    PRG,
    CHR,
}
impl ROMSizeMSB {
    fn get(self, arg: ROMSizeMSBEnum) -> u8 {
        match arg {
            ROMSizeMSBEnum::PRG => self.0 & 0x0f,
            ROMSizeMSBEnum::CHR => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PRGRAMEEPROMSize(u8);
enum PRGRAMEEPROMSizeEnum {
    PRG_RAM_SIZE,
    EEPROM_SIZE,
}
impl PRGRAMEEPROMSize {
    fn get(self, arg: PRGRAMEEPROMSizeEnum) -> u8 {
        match arg {
            PRGRAMEEPROMSizeEnum::PRG_RAM_SIZE => self.0 & 0x0f,
            PRGRAMEEPROMSizeEnum::EEPROM_SIZE => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CHRRAMSize(u8);
enum CHRRAMSizeEnum {
    CHR_RAM_SIZE,
    CHR_NVRAM_SIZE,
}
impl CHRRAMSize {
    fn get(self, arg: CHRRAMSizeEnum) -> u8 {
        match arg {
            CHRRAMSizeEnum::CHR_RAM_SIZE => self.0 & 0x0f,
            CHRRAMSizeEnum::CHR_NVRAM_SIZE => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Timing(u8);
impl Timing {
    fn get(self) -> u8 {
        self.0 & 0x03
    }
}

#[derive(Clone, Copy, Debug)]
struct VsSystemType(u8);
enum VsSystemTypeEnum {
    PPU_TYPE,
    HARDWARE_TYPE,
}
impl VsSystemType {
    fn get(self, arg: VsSystemTypeEnum) -> u8 {
        match arg {
            VsSystemTypeEnum::PPU_TYPE => self.0 & 0x0f,
            VsSystemTypeEnum::HARDWARE_TYPE => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ExtendedConsoleType(u8);
impl ExtendedConsoleType {
    fn get(self) -> u8 {
        self.0 & 0x0f
    }
}

#[derive(Clone, Copy, Debug)]
struct MiscROMs(u8);
impl MiscROMs {
    fn get(self) -> u8 {
        self.0 & 0x03
    }
}

#[derive(Clone, Copy, Debug)]
struct DefaultExpansionDevice(u8);
impl DefaultExpansionDevice {
    fn get(self) -> u8 {
        self.0 & 0x3f
    }
}

#[derive(Clone, Copy, Debug)]
enum ConsoleType {
    VsSystemType(VsSystemType),
    Extended(ExtendedConsoleType),
    _Unused(u8),
}

#[derive(Clone, Copy, Debug)]
pub struct Header {
    magic: [u8; 4],
    prg_rom_size_lsb: u8,
    chr_rom_size_lsb: u8,
    flags1: Flags1,
    flags2: Flags2,
    mapper_msb: MapperMSB,
    rom_size_msb: ROMSizeMSB,
    prg_ram_eeprom_size: PRGRAMEEPROMSize,
    chr_ram_size: CHRRAMSize,
    timing: Timing,
    console_type: ConsoleType,
    misc_roms: MiscROMs,
    default_expansion_device: DefaultExpansionDevice,
}

impl Header {
    fn new(bytes: [u8; 16]) -> Self {
        assert!(bytes[0..4] == NES_MAGIC);
        // assert!(bytes[7] & 0x0c == 0x08);
        Header {
            magic: NES_MAGIC,
            prg_rom_size_lsb: bytes[4],
            chr_rom_size_lsb: bytes[5],
            flags1: Flags1(bytes[6]),
            flags2: Flags2(bytes[7]),
            mapper_msb: MapperMSB(bytes[8]),
            rom_size_msb: ROMSizeMSB(bytes[9]),
            prg_ram_eeprom_size: PRGRAMEEPROMSize(bytes[10]),
            chr_ram_size: CHRRAMSize(bytes[11]),
            timing: Timing(bytes[12]),
            console_type: match bytes[7] & 0x03 {
                1 => ConsoleType::VsSystemType(VsSystemType(bytes[13])),
                3 => ConsoleType::Extended(ExtendedConsoleType(bytes[13])),
                _ => ConsoleType::_Unused(bytes[13]),
            },
            misc_roms: MiscROMs(bytes[14]),
            default_expansion_device: DefaultExpansionDevice(bytes[15]),
        }
    }
}

fn get_prg_rom_size(header: Header) -> usize {
    if header.rom_size_msb.get(ROMSizeMSBEnum::PRG) == 0xF {
        (2_u32.pow((header.prg_rom_size_lsb & 0xFC) as u32)
            * ((header.prg_rom_size_lsb & 0x03) * 2 + 1) as u32) as usize
    } else {
        ((header.prg_rom_size_lsb as u16
            | ((header.rom_size_msb.get(ROMSizeMSBEnum::PRG) as u16) << 8))
            * 16384) as usize
    }
}

fn get_chr_rom_size(header: Header) -> usize {
    if header.rom_size_msb.get(ROMSizeMSBEnum::CHR) == 0xF {
        (2_u32.pow((header.chr_rom_size_lsb & 0xFC) as u32)
            * ((header.chr_rom_size_lsb & 0x03) * 2 + 1) as u32) as usize
    } else {
        ((header.chr_rom_size_lsb as u16
            | ((header.rom_size_msb.get(ROMSizeMSBEnum::CHR) as u16) << 8))
            * 8192) as usize
    }
}

#[derive(Debug)]
pub struct File {
    // Header
    pub header: Header,

    // Trainer Area
    pub trainer: Option<[u8; 512]>,

    // PRG-ROM Area
    pub prg_rom_area: Vec<u8>,

    // CHR-ROM Area
    pub chr_rom_area: Option<Vec<u8>>,

    // Misc ROM Area
    pub misc_rom_area: Option<Vec<u8>>,
}

impl File {
    pub fn new(file_path: &str) -> Self {
        let bytes = std::fs::read(file_path).unwrap();
        let file_size = bytes.len();

        let header = Header::new(bytes[..16].try_into().unwrap());
        let (trainer, prg_rom_pos) = match header.flags1.get(Flags1Enum::TRAINER) {
            1 => (Some(bytes[16..528].try_into().unwrap()), 528),
            _ => (None, 16),
        };

        let prg_rom_size = get_prg_rom_size(header);
        let prg_rom_area = bytes[prg_rom_pos..prg_rom_pos + prg_rom_size].to_vec();

        let chr_rom_pos = prg_rom_pos + prg_rom_size;
        let chr_rom_size = get_chr_rom_size(header);

        let chr_rom_area = if chr_rom_size > 0 {
            Some(bytes[chr_rom_pos..chr_rom_pos + chr_rom_size].to_vec())
        } else {
            None
        };

        let misc_rom_pos = chr_rom_pos + chr_rom_size;
        let misc_rom_area = if misc_rom_pos < file_size {
            Some(bytes[misc_rom_pos..file_size].to_vec())
        } else {
            None
        };

        File {
            header,
            trainer,
            prg_rom_area,
            chr_rom_area,
            misc_rom_area,
        }
    }
}
