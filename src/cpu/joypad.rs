use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone)]
    pub struct Buttons: u8 {
        const RIGHT     = 0b10000000;
        const LEFT      = 0b01000000;
        const DOWN      = 0b00100000;
        const UP        = 0b00010000;
        const START     = 0b00001000;
        const SELECT    = 0b00000100;
        const BUTTON_B  = 0b00000010;
        const BUTTON_A  = 0b00000001;
    }
}

pub struct Joypad {
    strobe: bool,
    button_index: u8,
    pub buttons: Buttons,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            strobe: false,
            button_index: 0,
            buttons: Buttons::empty(),
        }
    }
}

impl Joypad {
    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        let response = (self.buttons.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }
        response
    }
}
