#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

const NES_MAGIC: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];

pub enum NameTableMirrorType {
    HORIZONTAL_OR_MAPPER,
    VERTICAL,
}

impl NameTableMirrorType {
    pub fn get(val: u8) -> Self {
        match val {
            0 => NameTableMirrorType::HORIZONTAL_OR_MAPPER,
            1 => NameTableMirrorType::VERTICAL,
            _ => panic!("Invalid NameTableMirrorType value"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Flags1(u8);

pub enum Flags1Enum {
    NAME_TABLE_MIRROR,
    BATTERY,
    TRAINER,
    FOUR_SCREEN_MODE,
    MAPPER_NUM,
}

impl Flags1 {
    pub fn get(&self, arg: Flags1Enum) -> u8 {
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
pub struct Flags2(u8);

pub enum Flags2Enum {
    CONSOLE_TYPE,
    MAGIC,
    MAPPER_NUM,
}

impl Flags2 {
    pub fn get(&self, arg: Flags2Enum) -> u8 {
        match arg {
            Flags2Enum::CONSOLE_TYPE => self.0 & 0x03,
            Flags2Enum::MAGIC => (self.0 & 0x0c) >> 2,
            Flags2Enum::MAPPER_NUM => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MapperMSB(u8);

pub enum MapperMSBEnum {
    MAPPER_NUM,
    SUBMAPPER_NUM,
}

impl MapperMSB {
    pub fn get(&self, arg: MapperMSBEnum) -> u8 {
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
    fn get(&self, arg: ROMSizeMSBEnum) -> u8 {
        match arg {
            ROMSizeMSBEnum::PRG => self.0 & 0x0f,
            ROMSizeMSBEnum::CHR => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PRGRAMEEPROMSize(u8);

pub enum PRGRAMEEPROMSizeEnum {
    PRG_RAM_SIZE,
    EEPROM_SIZE,
}

impl PRGRAMEEPROMSize {
    pub fn get(&self, arg: PRGRAMEEPROMSizeEnum) -> u8 {
        match arg {
            PRGRAMEEPROMSizeEnum::PRG_RAM_SIZE => self.0 & 0x0f,
            PRGRAMEEPROMSizeEnum::EEPROM_SIZE => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CHRRAMSize(u8);

pub enum CHRRAMSizeEnum {
    CHR_RAM_SIZE,
    CHR_NVRAM_SIZE,
}

impl CHRRAMSize {
    pub fn get(&self, arg: CHRRAMSizeEnum) -> u8 {
        match arg {
            CHRRAMSizeEnum::CHR_RAM_SIZE => self.0 & 0x0f,
            CHRRAMSizeEnum::CHR_NVRAM_SIZE => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Timing(u8);

impl Timing {
    pub fn get(&self) -> u8 {
        self.0 & 0x03
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VsSystemType(u8);

pub enum VsSystemTypeEnum {
    PPU_TYPE,
    HARDWARE_TYPE,
}

impl VsSystemType {
    pub fn get(&self, arg: VsSystemTypeEnum) -> u8 {
        match arg {
            VsSystemTypeEnum::PPU_TYPE => self.0 & 0x0f,
            VsSystemTypeEnum::HARDWARE_TYPE => (self.0 & 0xf0) >> 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExtendedConsoleType(u8);

impl ExtendedConsoleType {
    pub fn get(&self) -> u8 {
        self.0 & 0x0f
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MiscROMs(u8);

impl MiscROMs {
    pub fn get(&self) -> u8 {
        self.0 & 0x03
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DefaultExpansionDevice(u8);

impl DefaultExpansionDevice {
    pub fn get(&self) -> u8 {
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
    _magic: [u8; 4],
    prg_rom_size_lsb: u8,
    chr_rom_size_lsb: u8,
    pub flags1: Flags1,
    _flags2: Flags2,
    _mapper_msb: MapperMSB,
    rom_size_msb: ROMSizeMSB,
    prg_ram_eeprom_size: PRGRAMEEPROMSize,
    _chr_ram_size: CHRRAMSize,
    _timing: Timing,
    _console_type: ConsoleType,
    _misc_roms: MiscROMs,
    _default_expansion_device: DefaultExpansionDevice,
}

impl Header {
    pub fn new(bytes: [u8; 16]) -> Self {
        assert_eq!(bytes[0..4], NES_MAGIC);
        // assert!(bytes[7] & 0x0c == 0x08);
        Header {
            _magic: NES_MAGIC,
            prg_rom_size_lsb: bytes[4],
            chr_rom_size_lsb: bytes[5],
            flags1: Flags1(bytes[6]),
            _flags2: Flags2(bytes[7]),
            _mapper_msb: MapperMSB(bytes[8]),
            rom_size_msb: ROMSizeMSB(bytes[9]),
            prg_ram_eeprom_size: PRGRAMEEPROMSize(bytes[10]),
            _chr_ram_size: CHRRAMSize(bytes[11]),
            _timing: Timing(bytes[12]),
            _console_type: match bytes[7] & 0x03 {
                1 => ConsoleType::VsSystemType(VsSystemType(bytes[13])),
                3 => ConsoleType::Extended(ExtendedConsoleType(bytes[13])),
                _ => ConsoleType::_Unused(bytes[13]),
            },
            _misc_roms: MiscROMs(bytes[14]),
            _default_expansion_device: DefaultExpansionDevice(bytes[15]),
        }
    }
}

pub fn get_prg_rom_size(header: Header) -> usize {
    if header.rom_size_msb.get(ROMSizeMSBEnum::PRG) == 0xF {
        (2_u32.pow((header.prg_rom_size_lsb & 0xFC) as u32)
            * ((header.prg_rom_size_lsb & 0x03) * 2 + 1) as u32) as usize
    } else {
        let lsb = header.prg_rom_size_lsb as u16;
        let msb = header.rom_size_msb.get(ROMSizeMSBEnum::PRG) as u16;
        (msb << 8 | lsb) as usize * 16384
    }
}

pub fn get_chr_rom_size(header: Header) -> usize {
    let msb = header.rom_size_msb.get(ROMSizeMSBEnum::CHR) as usize;
    let lsb = header.chr_rom_size_lsb as usize;
    if header.rom_size_msb.get(ROMSizeMSBEnum::CHR) == 0xF {
        (2_u32.pow((lsb & 0xFC) as u32) * ((msb & 0x03) * 2 + 1) as u32) as usize
    } else {
        (lsb | (msb << 8)) * 8192
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

    pub fn get_prg_ram_size(&self) -> usize {
        let shift_count = self
            .header
            .prg_ram_eeprom_size
            .get(PRGRAMEEPROMSizeEnum::PRG_RAM_SIZE);
        if shift_count == 0 {
            return 0;
        }
        64 << shift_count
    }

    pub fn get_eeprom_size(&self) -> usize {
        let shift_count = self
            .header
            .prg_ram_eeprom_size
            .get(PRGRAMEEPROMSizeEnum::EEPROM_SIZE);
        if shift_count == 0 {
            return 0;
        }
        64 << shift_count
    }
}
