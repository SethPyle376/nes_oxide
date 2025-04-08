use crate::cpu::Mirroring;
use crate::ppu::registers::{MaskRegister, ScrollRegister, StatusRegister};
use registers::{AddressRegister, ControlRegister};

mod registers;

const CHR_ROM_BEGIN: u16 = 0;
const CHR_ROM_END: u16 = 0x01FF;
const VRAM_BEGIN: u16 = 0x2000;
const VRAM_END: u16 = 0x2FFF;
const PALETTE_BEGIN: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette_table: Vec<u8>,
    pub vram: Vec<u8>,
    pub oam_data: Vec<u8>,
    mirroring: Mirroring,
    data_buffer: u8,
    // Registers
    pub addr: AddressRegister,
    pub ctrl: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub scroll: ScrollRegister,
    oam_addr: u8,
    cycle: u64,
    scanline: u64,
    pub nmi_interrupt: Option<u8>,
}

impl Ppu {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Ppu {
            chr_rom,
            palette_table: vec![0; 32],
            vram: vec![0; 2048],
            oam_data: vec![0; 256],
            mirroring,
            data_buffer: 0,
            addr: AddressRegister::default(),
            ctrl: ControlRegister::default(),
            mask: MaskRegister::default(),
            status: StatusRegister::default(),
            scroll: ScrollRegister::default(),
            oam_addr: 0,
            cycle: 0,
            scanline: 0,
            nmi_interrupt: None,
        }
    }

    pub fn step(&mut self, cycles: u8) -> bool {
        self.cycle += cycles as u64;

        if self.cycle >= 341 {
            self.cycle -= 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status.set(StatusRegister::VBLANK_STARTED, true);
                self.status.set(StatusRegister::SPRITE_ZERO_HIT, false);
                if self.ctrl.contains(ControlRegister::GENERATE_NMI) {
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = None;
                self.status.set(StatusRegister::VBLANK_STARTED, false);
                self.status.set(StatusRegister::SPRITE_ZERO_HIT, true);
                return true;
            }
        }
        false
    }

    pub fn write_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_ctrl(&mut self, value: u8) {
        let nmi_status = self.ctrl.contains(ControlRegister::GENERATE_NMI);
        self.ctrl.update(value);

        if !nmi_status
            && self.ctrl.contains(ControlRegister::GENERATE_NMI)
            && self.status.contains(StatusRegister::VBLANK_STARTED)
        {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn write_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    pub fn read_status(&mut self) -> u8 {
        let data = self.status.bits();
        self.status.remove(StatusRegister::VBLANK_STARTED);
        self.scroll.latch = false;
        self.addr.high_byte = true;

        data
    }

    pub fn write_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_scroll(&mut self, value: u8) {
        self.scroll.update(value);
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();

        self.addr.increment(self.ctrl.vram_addr_increment());

        match addr {
            CHR_ROM_BEGIN..=CHR_ROM_END => {
                let result = self.data_buffer;
                self.data_buffer = self.chr_rom[addr as usize];
                result
            }
            VRAM_BEGIN..=VRAM_END => {
                let result = self.data_buffer;
                self.data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette_table[(addr - 0x3F10) as usize],
            PALETTE_BEGIN..=PALETTE_END => self.palette_table[Ppu::palette_mirror(addr) as usize],
            // _ => println!("unexpected ppu read address {}", addr)
            _ => 0,
        }
    }

    pub fn write_data(&mut self, value: u8) {
        let addr = self.addr.get();
        self.addr.increment(self.ctrl.vram_addr_increment());

        let mirrored_addr = self.mirror_vram_addr(addr);

        match addr {
            CHR_ROM_BEGIN..=CHR_ROM_END => println!("Attempt to write to CHR ROM"),
            VRAM_BEGIN..=VRAM_END => self.vram[mirrored_addr as usize] = value,
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette_table[(addr - 0x3F10) as usize] = value
            }
            PALETTE_BEGIN..=PALETTE_END => {
                self.palette_table[Ppu::palette_mirror(addr) as usize] = value
            }
            _ => println!("unexpected ppu write address {}", addr),
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }

    fn palette_mirror(addr: u16) -> u16 {
        let addr = addr & 0x001F;

        if addr >= 16 && addr.trailing_zeros() >= 2 {
            addr - 16
        } else {
            addr
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;

        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) | (Mirroring::Horizontal, 3) => {
                vram_index - 0x800
            }
            (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 2) => vram_index - 0x400,
            _ => vram_index,
        }
    }
}
