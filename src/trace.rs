use std::collections::HashMap;

use crate::cpu::{Memory, CPU};
use crate::{color, opcode};

pub fn trace(cpu: &CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPECODE_MAP;
    let pc_base = cpu.program_counter;

    let code = cpu.mem_read(pc_base);
    let ops = opcodes.get(&code).unwrap();

    let low_operand_str = if ops.len > 1 {
        format!("{:02X}", cpu.mem_read(pc_base + 1))
    } else {
        "  ".to_string()
    };

    let high_operand_str = if ops.len > 2 {
        format!("{:02X}", cpu.mem_read(pc_base + 2))
    } else {
        "  ".to_string()
    };

    let machine = format!(
        "{:02X} {:} {:}",
        ops.code, low_operand_str, high_operand_str
    );

    let operand = match ops.mode {
        crate::cpu::AddressingMode::Absolute => {
            format!("${:}{:}", high_operand_str, low_operand_str)
        }
        crate::cpu::AddressingMode::Immediate => {
            format!("#${:}", low_operand_str)
        }
        crate::cpu::AddressingMode::ZeroPage => {
            format!("${:} = 00", low_operand_str)
        }
        crate::cpu::AddressingMode::NoneAdressing => {
            if ops.len > 1 {
                let jump_to = pc_base + 2 + cpu.mem_read(pc_base + 1) as u16;
                format!("${:02X}", jump_to)
            } else {
                "".to_string()
            }
        }
        _ => format!("{:?}", ops.mode),
    };
    let asm = format!("{} {}", ops.mnemonic, operand);
    let asm = format!("{:27}", asm);

    let result = format!(
        "{:04X}  {:}  {:}     A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", // //{:?}",
        cpu.program_counter,
        machine,
        asm,
        cpu.register_a,
        cpu.register_x,
        cpu.register_y,
        cpu.status,
        cpu.stack_pointer,
        // ops.mode
    );
    return result;
}
