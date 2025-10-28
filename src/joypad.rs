use bitflags::bitflags;

bitflags! {
    pub struct JoypadButton: u8 {
        const RIGHT = 0b1000_0000;
    }
}
