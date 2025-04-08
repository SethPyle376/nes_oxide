use bitflags::bitflags;

pub struct AddressRegister {
    pub value: (u8, u8),
    pub high_byte: bool,
}

impl Default for AddressRegister {
    fn default() -> Self {
        AddressRegister {
            value: (0, 0),
            high_byte: true,
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

        if self.get() > 0x3fff {
            //mirror down addr above 0x3fff
            self.set(self.get() & 0b11111111111111);
        }

        self.high_byte = !self.high_byte;
    }

    fn set(&mut self, value: u16) {
        self.value.0 = (value >> 8) as u8;
        self.value.1 = (value & 0xff) as u8;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }

    pub fn increment(&mut self, value: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(value);

        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        if self.get() > 0x3fff {
            self.set(self.get() & 0b11111111111111);
        }
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
        *self = ControlRegister::from_bits_truncate(data);
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }

    pub fn sprite_pattern_addr(&self) -> u16 {
        if self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }
}

bitflags! {
  pub struct MaskRegister: u8 {
    const GREYSCALE = 1;
    const SHOW_LEFT_BACKGROUND = 1 << 1;
    const SHOW_LEFT_SPRITES = 1 << 2;
    const SHOW_BACKGROUND = 1 << 3;
    const SHOW_SPRITES = 1 << 4;
    const EMPHASIZE_RED = 1 << 5;
    const EMPHASIZE_GREEN = 1 << 6;
    const EMPHASIZE_BLUE = 1 << 7;
  }
}

impl Default for MaskRegister {
    fn default() -> Self {
        Self::from_bits_truncate(0)
    }
}

impl MaskRegister {
    pub fn update(&mut self, data: u8) {
        *self = MaskRegister::from_bits_truncate(data);
    }
}

bitflags! {
  pub struct StatusRegister: u8 {
    const UNUSED_1 = 1;
    const UNUSED_2 = 1 << 1;
    const UNUSED_3 = 1 << 2;
    const UNUSED_4 = 1 << 3;
    const UNUSED_5 = 1 << 4;
    const SPRITE_OVERFLOW = 1 << 5;
    const SPRITE_ZERO_HIT = 1 << 6;
    const VBLANK_STARTED = 1 << 7;
  }
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self::from_bits_truncate(0)
    }
}

impl StatusRegister {
    pub fn update(&mut self, data: u8) {
        *self = StatusRegister::from_bits_truncate(data);
    }
}

pub struct ScrollRegister {
    pub x: u8,
    pub y: u8,
    pub latch: bool,
}

impl Default for ScrollRegister {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            latch: false,
        }
    }
}

impl ScrollRegister {
    pub fn update(&mut self, data: u8) {
        if !self.latch {
            self.x = data;
        } else {
            self.y = data;
        }

        self.latch = !self.latch;
    }
}
