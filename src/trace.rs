use std::collections::HashMap;

use crate::cpu::{Memory, CPU};
use crate::opcode;

pub fn trace(cpu: &CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPECODE_MAP;

    let code = cpu.mem_read(cpu.program_counter);
    let ops = opcodes.get(&code).unwrap();

    let low_operand = if ops.len > 1 {
        format!("{:02X}", cpu.mem_read(cpu.program_counter + 1))
    } else {
        "  ".to_string()
    };

    let high_operand = if ops.len > 2 {
        format!("{:02X}", cpu.mem_read(cpu.program_counter + 2))
    } else {
        "  ".to_string()
    };

    println!(
        "{:04X}  {:02X} {:} {:}  {:} {:}     A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.program_counter,
        ops.code,
        low_operand,
        high_operand,
        ops.mnemonic,
        ops.len,
        cpu.register_a,
        cpu.register_x,
        cpu.register_y,
        cpu.status,
        cpu.stack_pointer
    );
    return "".to_owned();
}
