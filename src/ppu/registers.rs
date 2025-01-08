use bitflags::bitflags;

pub struct AddressRegister {
  value: (u8, u8),
  high_byte: bool
}

impl Default for AddressRegister {
  fn default() -> Self {
    AddressRegister {
      value: (0, 0),
      high_byte: true
    }
  }
}

impl AddressRegister {
  pub fn update(&mut self, data: u8) {
    if self.high_byte {
      self.value.0 = data;
    } else {
      self.value.1 = data;
    }

    self.high_byte = !self.high_byte;
  }

  pub fn get(&self) -> u16 {
    let value = ((self.value.0 as u16) << 8) | self.value.1 as u16;

    return value & 0x3FFF;
  }

  pub fn increment(&mut self, value: u8) {
    let sum = self.get().wrapping_add(value as u16) & 0x3FFF;

    self.value.0 = (sum >> 8) as u8;
    self.value.1 = (sum & 0xFF) as u8;
  }
}

bitflags! {
  pub struct ControlRegister: u8 {
    const NAMETABLE1 = 1;
    const NAMETABLE2 = 1 << 1;
    const VRAM_ADD_INCREMENT = 1 << 2;
    const SPRITE_PATTERN_ADDR = 1 << 3;
    const BACKGROUND_PATTERN_ADDR = 1 << 4;
    const SPRITE_SIZE = 1 << 5;
    const MASTER_SLAVE_SELECT = 1 << 6;
    const GENERATE_NMI = 1 << 7;
  }
}

impl Default for ControlRegister {
  fn default() -> Self {
    ControlRegister::from_bits_truncate(0)
  }
}

impl ControlRegister {
  pub fn vram_addr_increment(&self) -> u8 {
    if self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
      32
    } else {
      1
    }
  }

  pub fn update(&mut self, data: u8) {
    self.update(data);
  }
}