pub struct Bus {
    ram: Vec<u8>,
    // cartridge
    // ppu
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            ram: vec![0; 0x800]
        }
    }
}

const RAM_BEGIN: u16    = 0x0000;
const RAM_END: u16      = 0x1FFF;

impl Bus {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_BEGIN ..= RAM_END => {
                return self.ram[usize::from(addr & 0x7FF)];
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            RAM_BEGIN ..= RAM_END => {
                self.ram[usize::from(addr & 0x7FF)] = value;
            }
            _ => {
                todo!()
            }
        }
    }
}
