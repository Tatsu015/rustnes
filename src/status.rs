use bitflags::bitflags;

bitflags! {
    pub struct StatusRegister:u8{
        const SPRITE_OVERFLOW_FLAG = 0b0010_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const VBLANK_STARTED = 0b1000_0000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0)
    }

    pub fn set_start_vblank(&mut self) {
        self.set(StatusRegister::VBLANK_STARTED, true);
    }

    pub fn set_end_vblank(&mut self) {
        self.set(StatusRegister::VBLANK_STARTED, false);
    }

    pub fn is_in_vbrank(&self) -> bool {
        self.contains(StatusRegister::VBLANK_STARTED)
    }

    pub fn snapshot(&self) -> u8 {
        self.bits()
    }
}
