const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const HEADER_LENGTH: usize = 16;
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Clone, Copy, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
}

impl Cartridge {
    pub fn new(bytes: &Vec<u8>) -> Result<Cartridge, String> {
        if &bytes[0..4] != NES_TAG {
            return Err("FILE IS NOT AN iNES ROM".to_string());
        }

        // Mapper byte contained in top half of bytes 6 and 7
        let mapper = (bytes[7] & 0xF0) | (bytes[6] >> 4);

        // iNES version info is in bits 2 & 3 of byte 7
        let ines_version = (bytes[7] >> 2) & 0x03;

        if ines_version == 2 {
            return Err("iNES VERSION 2 IS NOT SUPPORTED".to_string());
        } else if ines_version != 0 {
            return Err("UNSUPPORTED iNES VERSION DETECTED".to_string());
        }

        // Four screen info is bit 3 of byte 6
        let four_screen = bytes[6] & 0x08 != 0;

        // Vertical mirroring is bit 0 of byte 6
        let vertical_mirroring = bytes[6] & 0x01 != 0;

        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_length = bytes[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_length = bytes[5] as usize * CHR_ROM_PAGE_SIZE;

        // If byte 6 bit 2 is true there is a 512 byte block between the HEADER and PRG_ROM
        let trainer_length = if bytes[6] & 0x04 == 1 {
            512 as usize
        } else {
            0 as usize
        };

        let prg_rom_start = HEADER_LENGTH + trainer_length;
        let chr_rom_start = prg_rom_start + prg_rom_length;

        let prg_rom = bytes[prg_rom_start..(prg_rom_start + prg_rom_length)].to_vec();
        let chr_rom = bytes[chr_rom_start..(chr_rom_start + chr_rom_length)].to_vec();

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            mapper,
            mirroring,
        })
    }

    pub fn load(path: &str) -> Result<Cartridge, String> {
        return Cartridge::new(&std::fs::read(path).unwrap());
    }
}
