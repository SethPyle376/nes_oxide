pub enum MappedRead {
    PrgRom(u16),
    PrgRam(u16),
    ChrRom(u16),
    ChrRam(u16),
}

pub enum MappedWrite {
    PrgRom(u16),
    PrgRam(u16),
    ChrRom(u16),
    ChrRam(u16),
}

pub trait Mapper {
    fn cpu_read(addr: u16) -> Option<MappedRead>;
    fn cpu_write(addr: u16);
}
