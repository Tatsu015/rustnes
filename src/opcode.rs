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
        // AND
        OpCode::new(0x29, "AND", 2, 2, Immidiate),
        OpCode::new(0x25, "AND", 2, 3, ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, ZeroPage_X),
        OpCode::new(0x2d, "AND", 3, 4, Absolute),
        OpCode::new(0x3d, "AND", 3, 4 /*(+1 if page crossed)*/, Absolute_X),
        OpCode::new(0x39, "AND", 3, 4 /*(+1 if page crossed)*/, Absolute_Y),
        OpCode::new(0x21, "AND", 2, 6, Indirect_X),
        OpCode::new(0x31, "AND", 2, 5 /*(+1 if page crossed)*/, Indirect_Y),
        // ASL
        OpCode::new(0x29, "ASL", 1, 2, NoneAdressing),
        OpCode::new(0x25, "ASL", 2, 5, ZeroPage),
        OpCode::new(0x35, "ASL", 2, 6, ZeroPage_X),
        OpCode::new(0x2d, "ASL", 3, 6, Absolute),
        OpCode::new(0x3d, "ASL", 3, 7, Absolute_X),
        // BCC
        OpCode::new(0x90, "BCC", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BCS
        OpCode::new(0xb0, "BCS", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BEQ
        OpCode::new(0xf0, "BEQ", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BIT
        OpCode::new(0x24, "BIT", 2, 5, ZeroPage),
        OpCode::new(0x2c, "BIT", 2, 6, Absolute),
        // BMI
        OpCode::new(0x30, "BMI", 2, 2, /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BNE
        OpCode::new(0xd0, "BNE", 2, 2, /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BPL
        OpCode::new(0x10, "BPL", 2, 2, /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BRK
        OpCode::new(0x00, "BRK", 1, 7, NoneAdressing),
        // BVC
        OpCode::new(0xd0, "BNE", 2, 2, /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // BVS
        OpCode::new(0x70, "BVS", 2, 2, /*(+1 if branch succeeds +2 if to a new page)*/, NoneAdressing),
        // CLC
        OpCode::new(0x18, "CVC", 1, 2, , NoneAdressing),
        // CLD
        OpCode::new(0xd8, "CLD", 1, 2, , NoneAdressing),
        // CLI
        OpCode::new(0x58, "CLD", 1, 2, , NoneAdressing),
        // CLV
        OpCode::new(0xb8, "CLD", 1, 2, , NoneAdressing),
        // CMP
        OpCode::new(0xc9, "CMP", 2, 2, Immidiate),
        OpCode::new(0xc5, "CMP", 2, 3, ZeroPage),
        OpCode::new(0xd5, "CMP", 2, 4, ZeroPage_X),
        OpCode::new(0xcd, "CMP", 3, 4, Absolute),
        OpCode::new(0xdd, "CMP", 3, 4 /*(+1 if page crossed)*/, Absolute_X),
        OpCode::new(0xd9, "CMP", 3, 4 /*(+1 if page crossed)*/, Absolute_Y),
        OpCode::new(0xc1, "CMP", 2, 6, Indirect_X),
        OpCode::new(0xd1, "CMP", 2, 5 /*(+1 if page crossed)*/, Indirect_Y),
        // CPX
        OpCode::new(0xe0, "CPX", 2, 2, Immidiate),
        OpCode::new(0xe4, "CPX", 2, 3, ZeroPage),
        OpCode::new(0xec, "CPX", 3, 4, Absolute),
        // CPY
        OpCode::new(0xc0, "CPY", 2, 2, Immidiate),
        OpCode::new(0xc4, "CPY", 2, 3, ZeroPage),
        OpCode::new(0xcc, "CPY", 3, 4, Absolute),
        // DEC
        OpCode::new(0xc6, "DEC", 2, 5, ZeroPage),
        OpCode::new(0xd6, "DEC", 2, 6, ZeroPage_X),
        OpCode::new(0xce, "DEC", 3, 6, Absolute),
        OpCode::new(0xde, "DEC", 3, 7, Absolute_X),
        // DEX
        OpCode::new(0xca, "DEX", 1, 2, NoneAdressing),
        // DEY
        OpCode::new(0x88, "DEY", 1, 2, NoneAdressing),
        // EOR
        OpCode::new(0x49, "EOR", 2, 2, Immidiate),
        OpCode::new(0x45, "EOR", 2, 3, ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, ZeroPage_X),
        OpCode::new(0x4d, "EOR", 3, 4, Absolute),
        OpCode::new(0x5d, "EOR", 3, 4 /*(+1 if page crossed)*/, Absolute_X),
        OpCode::new(0x59, "EOR", 3, 4 /*(+1 if page crossed)*/, Absolute_Y),
        OpCode::new(0x41, "EOR", 2, 6, Indirect_X),
        OpCode::new(0x51, "EOR", 2, 5 /*(+1 if page crossed)*/, Indirect_Y),
        // INC
        OpCode::new(0xe6, "INC", 2, 5, ZeroPage),
        OpCode::new(0xf6, "INC", 2, 6, ZeroPage_X),
        OpCode::new(0xee, "INC", 3, 6, Absolute),
        OpCode::new(0xfe, "INC", 3, 7, Absolute_X),
        // INX
        OpCode::new(0xe8, "INX", 1, 2, ZeroPage),
        // INY
        OpCode::new(0xc8, "INY", 1, 2, ZeroPage),
        // JMP
        OpCode::new(0x4c, "JMP", 3, 3, Absolute),
        OpCode::new(0x6c, "JMP", 3, 5, NoneAdressing),


        // BRK
        OpCode::new(0x00, "BRK", 1, 7, NoneAdressing),
    ];
}
