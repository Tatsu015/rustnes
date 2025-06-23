use std::collections::HashMap;

use crate::cpu::{Memory, CPU};
use crate::opcode;

pub fn trace(cpu: &CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPECODE_MAP;

    let code = cpu.mem_read(cpu.program_counter);
    let ops = opcodes.get(&code).unwrap();

    println!(
        "{:04X} {:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.program_counter,
        ops.code,
        cpu.register_a,
        cpu.register_x,
        cpu.register_y,
        cpu.status,
        cpu.stack_pointer
    );
    return "".to_owned();
}
