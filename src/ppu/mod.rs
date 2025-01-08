use crate::cpu::Mirroring;
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
  addr: AddressRegister,
  ctrl: ControlRegister,
}

impl Ppu {
  pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
    Ppu {
      chr_rom: chr_rom,
      palette_table: Vec::with_capacity(32),
      vram: Vec::with_capacity(2048),
      oam_data: Vec::with_capacity(256),
      mirroring,
      data_buffer: 0,
      addr: AddressRegister::default(),
      ctrl: ControlRegister::default()
    }
  }

  pub fn write_addr(&mut self, value: u8) {
    self.addr.update(value);
  }

  pub fn write_ctrl(&mut self, value: u8) {
    self.ctrl.update(value);
  }

  pub fn read_data(&mut self) -> u8 {
    let addr = self.addr.get();

    self.addr.increment(self.ctrl.vram_addr_increment());

    match addr {
      CHR_ROM_BEGIN..=CHR_ROM_END => {
        let result = self.data_buffer;
        self.data_buffer = self.chr_rom[addr as usize];
        result
      },
      VRAM_BEGIN..=VRAM_END => {
        let result = self.data_buffer;
        self.data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
        result
      },
      0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
        self.palette_table[(addr - 0x3F10) as usize]
      },
      PALETTE_BEGIN..=PALETTE_END => self.palette_table[(addr - 0x3f00) as usize], 
        _ => panic!("unexpected ppu read address {}", addr)
      }
  }

  fn mirror_vram_addr(&self, addr: u16) -> u16 {
    let mirrored_vram = addr & 0x2FFF;
    let vram_index = mirrored_vram - 0x2000;
    let name_table = vram_index / 0x400;

    match (&self.mirroring, name_table) {
      (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) | (Mirroring::Horizontal, 3) => vram_index - 0x800,
      (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 2) => vram_index - 0x400,
      _ => vram_index
    }
  }

}