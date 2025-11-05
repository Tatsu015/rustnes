use bitflags::bitflags;

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE1 = 0b0000_0001;
        const NAMETABLE2 = 0b0000_0010;
        const VRAM_ADDR_INCREMENT = 0b0000_0100;
        const SPRITE_PATTERN_ADDR = 0b0000_1000;
        const BACKGROUND_PATTERN_ADDR = 0b0001_0000;
        const STRIPE_SIZE = 0b0010_0000;
        const MASTER_SLAVE_SELECT = 0b0100_0000;
        const GENERATE_NMI = 0b1000_0000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADDR_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        *self = ControlRegister::from_bits_truncate(data);
    }

    pub fn generate_vblank_status(&mut self) -> bool {
        let result = self.contains(ControlRegister::GENERATE_NMI);
        self.set(ControlRegister::GENERATE_NMI, true);
        // println!("generate_vbrank_status result:{}", result); // TODO
        result
    }

    pub fn bknd_pattern_addr(&self) -> u16 {
        if self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }

    pub fn sprt_pattern_addr(&self) -> u16 {
        if self.contains(ControlRegister::STRIPE_SIZE) {
            0x1000
        } else {
            0
        }
    }
}
