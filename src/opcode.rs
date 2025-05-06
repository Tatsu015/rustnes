use AddressingMode::Absolute;
use AddressingMode::Absolute_X;
use AddressingMode::Absolute_Y;
use AddressingMode::Immediate;
use AddressingMode::Indirect_X;
use AddressingMode::Indirect_Y;
use AddressingMode::NoneAdressing;
use AddressingMode::ZeroPage;
use AddressingMode::ZeroPage_X;
use AddressingMode::ZeroPage_Y;

pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub len: u8,
    pub cycle: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    pub fn new(code: u8, mnemonic: &'static str, len: u8, cycle: u8, mode: AddressingMode) -> Self {
        OpCode {
            code,
            mnemonic,
            len,
            cycle,
            mode,
        }
    }
}

lazy_static! {
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        // ADC
        OpCode::new(0x69, "ADC", 2, 2, Immidiate),
        OpCode::new(0x65, "ADC", 2, 3, ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, ZeroPage_X),
        OpCode::new(0x6d, "ADC", 3, 4, Absolute),
        OpCode::new(0x7d, "ADC", 3, 4 /*(+1 if page crossed)*/, Absolute_X),
        OpCode::new(0x79, "ADC", 3, 4 /*(+1 if page crossed)*/, Absolute_Y),
        OpCode::new(0x61, "ADC", 2, 6, Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5 /*(+1 if page crossed)*/, Indirect_Y),

        // BRK
        OpCode::new(0x00, "BRK", 1, 7, NoneAdressing),
    ];
}
