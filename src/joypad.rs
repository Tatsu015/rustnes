use bitflags::bitflags;

bitflags! {
    pub struct JoypadButton: u8 {
        const RIGHT    = 0b1000_0000;
        const LEFT     = 0b0100_0000;
        const DOWN     = 0b0010_0000;
        const UP       = 0b0001_0000;
        const START    = 0b0000_1000;
        const SELECT   = 0b0000_0100;
        const BUTTON_B = 0b0000_0010;
        const BUTTON_A = 0b0000_0001;
    }
}

pub struct Joypad {
    strobe: bool,
    button_index: u8,
    button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::from_bits_truncate(0),
        }
    }
}
