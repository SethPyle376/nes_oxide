use crate::ppu::Ppu;

use super::Cartridge;

// RAM Addresses
const RAM_BEGIN: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

// PPU Register Addresses
const PPU_REGISTER_BEGIN: u16 = 0x2000;
const PPU_REGISTER_END: u16 = 0x3FFF;

const PPU_MAP_ADDR: u16 = 0x2006;
const PPU_MAP_READ: u16 = 0x2007;

const PRG_ROM_BEGIN: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus {
    pub ram: Vec<u8>,
    pub cartridge: Cartridge,
    pub ppu: Ppu
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Bus {
        let mut bus = Bus {
            ram: Vec::with_capacity(0x800),
            ppu: Ppu::new(cartridge.chr_rom.clone(), cartridge.mirroring),
            cartridge,
        };
        bus.ram.resize(0x800, 0x00);
        return bus;
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // Main RAM read
            RAM_BEGIN..=RAM_END => {
                return self.ram[usize::from(addr & 0x7FF)];
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                println!("ATTEMPTED TO READ WRITE ONLY PPU ADDRESS {:04x}", addr);
                return 0;
            }
            PPU_MAP_READ => self.ppu.read_data(),
            0x2008..=PPU_REGISTER_END => {
                // Mirror down address to real PPU space
                return self.read(addr & 0x2007);
            }
            PRG_ROM_BEGIN..=PRG_ROM_END => {
                let mut rom_location = addr - 0x8000;

                if self.cartridge.prg_rom.len() == 0x4000 {
                    rom_location = rom_location % 0x4000;
                }

                return self.cartridge.prg_rom[rom_location as usize];
            }
            _ => {
                // println!("IGNORING MEMORY READ AT ADDRESS {:04x}", addr);
                return 0;
            }
        }
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lsb = self.read(addr);
        let msb = self.read(addr.wrapping_add(1));
        u16::from_le_bytes([lsb, msb])
    }

    pub fn read_u16_zp(&mut self, addr: u8) -> u16 {
        let lo = self.read(addr.into());
        let hi = self.read(addr.wrapping_add(1).into());
        u16::from_le_bytes([lo, hi])
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            RAM_BEGIN..=RAM_END => {
                self.ram[usize::from(addr & 0x7FF)] = value;
            }
            PPU_MAP_ADDR => {}
            0x2008..=PPU_REGISTER_END => {
                // Mirror down address to real PPU space
                return self.write(addr & 0x2007, value);
            }
            PRG_ROM_BEGIN..=PRG_ROM_END => {
                println!("WRITE TO PRG ROM ATTEMPTED");
            }
            _ => {
                println!("IGNORING MEMORY WRITE AT ADDRESS {:04x}", addr);
            }
        }
    }

    pub fn get_page(&self, page: u8) -> &[u8] {
        let bounded = page & 0x7;
        let start = bounded as usize * 256;
        let end = start + 256;

        return &self.ram[start..end];
    }
}
