use crate::cpu::CPU;

pub fn trace(cpu: &CPU) -> String {
    println!(
        "{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.program_counter,
        cpu.register_a,
        cpu.register_x,
        cpu.register_y,
        cpu.status,
        cpu.stack_pointer
    );
    return "test".to_owned();
}
