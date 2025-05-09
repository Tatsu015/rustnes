use crate::opcode::{self, OpCode};
use std::{collections::HashMap, os::unix::process};

use bitflags::bitflags;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAdressing,
}

bitflags! {
    #[derive(Debug, Clone)]
    pub struct CpuFlags:u8 {
        const CARRY = 0b0000_0001;
        const ZERO = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const DECIMAL = 0b0000_1000;
        const BREAK=0b0001_0000;
        const RESERVED=0b0010_0000;
        const OVERFLOW = 0b0100_0000;
        const NEGATIVE = 0b1000_0000;
    }
}
const INITIAL_STATUS: u8 = CpuFlags::RESERVED.bits() | CpuFlags::INTERRUPT_DISABLE.bits();
const STACK: u16 = 0x0100;
const INITIAL_STACK: u8 = 0xfd;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub memory: [u8; 0xffff],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(INITIAL_STATUS),
            program_counter: 0,
            stack_pointer: INITIAL_STACK,
            memory: [0; 0xffff],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xfffc, 0x8000);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = INITIAL_STACK;
        self.status = CpuFlags::from_bits_truncate(INITIAL_STATUS);

        self.program_counter = self.mem_read_u16(0xfffc);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        let ref opcodes: HashMap<u8, &'static OpCode> = *opcode::OPECODE_MAP;
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let before_program_counter = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));

            match code {
                0xa9 => {
                    self.lda(&opcode.mode);
                }
                0xa5 => {
                    self.lda(&opcode.mode);
                }
                0xad => {
                    self.lda(&opcode.mode);
                }
                0x85 => {
                    self.sta(&opcode.mode);
                }
                0x95 => {
                    self.sta(&opcode.mode);
                }
                0xaa => self.tax(),
                0xe8 => self.inx(),
                0x00 => return, // BRK
                _ => todo!(""),
            }
            if before_program_counter == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }

    fn get_operand_adress(&self, mode: &AddressingMode) -> u16 {
        match mode {
            // Immidiate use program counter value as operand adress
            AddressingMode::Immediate => self.program_counter,
            // `page` is 256byte memory region.
            // for ex. 0page:0x0000 ~ 0x00ff, 1page:0x0100 ~ 0x01ff, ...
            // ZeroPage adressing uses only the first 256 bytes of memory, where the adress is in the instruction
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                let addr = (hi as u16) << 8 | lo as u16;
                addr
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | lo as u16;
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::NoneAdressing => {
                panic!("mode {:?} is not supported", mode)
            }
        }
    }

    #[allow(dead_code)]
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let carry = if self.status.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        };
        let overable_resule = self.register_a as i16 + data as i16 + carry as i16;
        if overable_resule > 0xff {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }
        let result = overable_resule as u8;
        if (result ^ self.register_a) & (result ^ data) & 0x80 == 0 {
            self.status.remove(CpuFlags::OVERFLOW);
        } else {
            self.status.insert(CpuFlags::OVERFLOW);
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(result);
    }

    #[allow(dead_code)]
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_a = self.register_a & self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[allow(dead_code)]
    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr) * 2;

        if self.register_a >> 7 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.mem_write(addr, data);
    }

    #[allow(dead_code)]
    fn asl_accumulator(&mut self) {
        if self.register_a >> 7 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.register_a = self.register_a * 2;
    }

    #[allow(dead_code)]
    fn bcc(&mut self) {
        self.branch(self.status.contains(CpuFlags::CARRY));
    }

    #[allow(dead_code)]
    fn bcs(&mut self) {
        self.branch(!self.status.contains(CpuFlags::CARRY));
    }

    #[allow(dead_code)]
    fn beq(&mut self) {
        self.branch(self.status.contains(CpuFlags::ZERO));
    }

    #[allow(dead_code)]
    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let and = self.register_a & data;
        if and == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        self.status
            .set(CpuFlags::NEGATIVE, data & CpuFlags::NEGATIVE.bits() > 0);
        self.status
            .set(CpuFlags::OVERFLOW, data & CpuFlags::OVERFLOW.bits() > 0);
    }

    #[allow(dead_code)]
    fn bmi(&mut self) {
        self.branch(self.status.contains(CpuFlags::NEGATIVE));
    }

    #[allow(dead_code)]
    fn bne(&mut self) {
        self.branch(!self.status.contains(CpuFlags::ZERO));
    }

    #[allow(dead_code)]
    fn bpl(&mut self) {
        self.branch(!self.status.contains(CpuFlags::NEGATIVE));
    }

    // no brk function needed. 0x00 case only return

    #[allow(dead_code)]
    fn bvc(&mut self) {
        self.branch(!self.status.contains(CpuFlags::OVERFLOW));
    }

    #[allow(dead_code)]
    fn bvs(&mut self) {
        self.branch(self.status.contains(CpuFlags::OVERFLOW));
    }

    #[allow(dead_code)]
    fn clc(&mut self) {
        self.status.remove(CpuFlags::CARRY);
    }

    #[allow(dead_code)]
    fn cld(&mut self) {
        self.status.remove(CpuFlags::DECIMAL);
    }

    #[allow(dead_code)]
    fn cli(&mut self) {
        self.status.remove(CpuFlags::INTERRUPT_DISABLE);
    }

    #[allow(dead_code)]
    fn clv(&mut self) {
        self.status.remove(CpuFlags::OVERFLOW);
    }

    #[allow(dead_code)]
    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        if self.register_a >= data {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(data));
    }

    #[allow(dead_code)]
    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        if self.register_x >= data {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_x.wrapping_sub(data));
    }

    #[allow(dead_code)]
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        if self.register_y >= data {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_y.wrapping_sub(data));
    }

    #[allow(dead_code)]
    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let new_data = data.wrapping_sub(1);
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
    }

    #[allow(dead_code)]
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[allow(dead_code)]
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[allow(dead_code)]
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        self.register_a = self.register_a ^ data;
        self.update_zero_and_negative_flags(self.register_a); // [TODO] maybe need.
    }

    #[allow(dead_code)]
    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let new_data = data.wrapping_add(1);
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
    }

    #[allow(dead_code)]
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[allow(dead_code)]
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[allow(dead_code)]
    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read_u16(addr);
        self.program_counter = data;
    }

    #[allow(dead_code)]
    fn jsr(&mut self) {
        let pc = self.program_counter - 1;
        self.stack_push_u16(pc);
        let adress = self.mem_read_u16(self.program_counter);
        self.program_counter = adress;
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_a = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[allow(dead_code)]
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_x = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[allow(dead_code)]
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_y = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[allow(dead_code)]
    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        if data & 1 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        let new_data = data >> 1;
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
    }

    #[allow(dead_code)]
    fn lsr_accumulator(&mut self) {
        if self.register_a & 1 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.register_a = self.register_a >> 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[allow(dead_code)]
    fn nop(&mut self) {
        // nothing to do
    }

    #[allow(dead_code)]
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        self.register_a = data | self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[allow(dead_code)]
    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    #[allow(dead_code)]
    fn php(&mut self) {
        let mut data = self.status.clone();
        data.insert(CpuFlags::BREAK);
        data.insert(CpuFlags::RESERVED);
        self.stack_push(data.bits());
    }

    #[allow(dead_code)]
    fn pla(&mut self) {
        self.register_a = self.stack_pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[allow(dead_code)]
    fn plp(&mut self) {
        self.status = CpuFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::RESERVED);
    }

    #[allow(dead_code)]
    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let mut data = self.mem_read(addr);
        let carry = self.status.contains(CpuFlags::CARRY);
        let bit7 = data << 7;
        if bit7 == 0 {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        data = data << 1;
        data = data | (carry as u8);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    #[allow(dead_code)]
    fn rol_accumulate(&mut self) {
        let mut data = self.register_a;
        let carry = self.status.contains(CpuFlags::CARRY);
        let bit7 = data << 7;
        if bit7 == 0 {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }
        data = data << 1;
        data = data | (carry as u8);
        self.register_a = data;
        self.update_zero_and_negative_flags(data);
    }

    #[allow(dead_code)]
    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let mut data = self.mem_read(addr);
        let carry = self.status.contains(CpuFlags::CARRY);
        let bit0 = data & 1;
        if bit0 == 0 {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        data = data >> 1;
        data = data | (carry as u8) << 7;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    #[allow(dead_code)]
    fn ror_accumulator(&mut self) {
        let mut data = self.register_a;
        let carry = self.status.contains(CpuFlags::CARRY);
        let bit0 = data & 1;
        if bit0 == 0 {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        data = data >> 1;
        data = data | (carry as u8) << 7;
        self.register_a = data;
        self.update_zero_and_negative_flags(data);
    }

    #[allow(dead_code)]
    fn rti(&mut self) {
        self.status = CpuFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::RESERVED);

        self.program_counter = self.stack_pop_u16();
    }

    #[allow(dead_code)]
    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    #[allow(dead_code)]
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let carry = if self.status.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        };
        let overable_result = self.register_a as i16 - (data as i16) + carry as i16;
        if overable_result > 0xff {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }
        let result = overable_result as u8;

        if (result ^ self.register_a) & (result ^ data) & 0x80 == 0 {
            self.status.remove(CpuFlags::OVERFLOW);
        } else {
            self.status.insert(CpuFlags::OVERFLOW);
        }

        self.register_a = result;
        self.update_zero_and_negative_flags(result);
    }

    #[allow(dead_code)]
    fn sec(&mut self) {
        self.status.insert(CpuFlags::CARRY);
    }

    #[allow(dead_code)]
    fn sed(&mut self) {
        self.status.insert(CpuFlags::DECIMAL);
    }

    #[allow(dead_code)]
    fn sei(&mut self) {
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.mem_write(addr, self.register_a);
    }

    #[allow(dead_code)]
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.mem_write(addr, self.register_x);
    }

    #[allow(dead_code)]
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.mem_write(addr, self.register_y);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[allow(dead_code)]
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[allow(dead_code)]
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[allow(dead_code)]
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[allow(dead_code)]
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    #[allow(dead_code)]
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            let jump = self.mem_read(self.program_counter);
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);
            self.program_counter = jump_addr;
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }
        if result & 0b1000_0000 != 0 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    #[allow(dead_code)]
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        let _ = self.stack_pointer.wrapping_sub(1);
    }

    #[allow(dead_code)]
    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        hi << 8 | lo
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO))
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_5_ops_working_togather() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55)
    }
}
