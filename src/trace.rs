use crate::cpu::CPU;

pub fn trace(cpu: &CPU) -> String {
    println!("{}", cpu.program_counter);
    return "test".to_owned();
}
