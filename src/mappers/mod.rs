pub mod nrom;

pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

pub trait Mapper {
    fn get_mirroring(&self) -> Mirroring;

    fn get_chr_rom(&self) -> &[u8];

    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);

    fn read_16(&self, addr: u16) -> u16 {
        let low = self.read(addr);
        let high = self.read(addr + 1);
        (high as u16) << 8 | low as u16
    }

    fn write_16(&mut self, addr: u16, data: u16) {
        self.write(addr, data as u8);
        self.write(addr + 1, (data >> 8) as u8);
    }
}
