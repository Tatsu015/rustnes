use crate::opcode::{self, OpCode};
use core::fmt;
use std::collections::HashMap;

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

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CPU")
            .field("register_a", &self.register_a)
            .field("register_x", &self.register_x)
            .field("register_y", &self.register_y)
            .field("status", &self.status)
            .field("program_counter", &self.program_counter)
            .field("stack_pointer", &self.stack_pointer)
            // .field("memory", &self.memory) // memoryフィールドは省略
            .finish()
    }
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

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
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
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static OpCode> = *opcode::OPECODE_MAP;
        loop {
            callback(self);

            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let before_program_counter = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));

            self.debug(code);

            match code {
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => self.adc(&opcode.mode),
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => self.and(&opcode.mode),
                0x0a => self.asl_accumulator(),
                0x06 | 0x16 | 0x0e | 0x1e => self.asl(&opcode.mode),
                0x90 => self.bcc(),
                0xb0 => self.bcs(),
                0xf0 => self.beq(),
                0x24 | 0x2c => self.bit(&opcode.mode),
                0x30 => self.bmi(),
                0xd0 => self.bne(),
                0x10 => self.bpl(),
                0x00 => return, // BRK
                0x50 => self.bvc(),
                0x70 => self.bvs(),
                0x18 => self.clc(),
                0xd8 => self.cld(),
                0x58 => self.cli(),
                0xb8 => self.clv(),
                0xd1 | 0xc1 | 0xd9 | 0xdd | 0xcd | 0xd5 | 0xc5 | 0xc9 => self.cmp(&opcode.mode),
                0xe0 | 0xe4 | 0xec => self.cpx(&opcode.mode),
                0xc0 | 0xc4 | 0xcc => self.cpy(&opcode.mode),
                0xc6 | 0xd6 | 0xce | 0xde => self.dec(&opcode.mode),
                0xca => self.dex(),
                0x88 => self.dey(),
                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => self.eor(&opcode.mode),
                0xe6 | 0xf6 | 0xee | 0xfe => self.inc(&opcode.mode),
                0xe8 => self.inx(),
                0xc8 => self.iny(),
                0x4c | 0x6c => self.jmp(&opcode.mode),
                0x20 => self.jsr(),
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.lda(&opcode.mode),
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.ldx(&opcode.mode),
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.ldy(&opcode.mode),
                0x4a => self.lsr_accumulator(),
                0x46 | 0x56 | 0x4e | 0x5e => self.lsr(&opcode.mode),
                0xea => self.nop(),
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => self.ora(&opcode.mode),
                0x48 => self.pha(),
                0x08 => self.php(),
                0x68 => self.pla(),
                0x28 => self.plp(),
                0x2a => self.rol_accumulate(),
                0x26 | 0x36 | 0x2e | 0x3e => self.rol(&opcode.mode),
                0x6a => self.ror_accumulator(),
                0x66 | 0x76 | 0x6e | 0x7e => self.ror(&opcode.mode),
                0x40 => self.rti(),
                0x60 => self.rts(),
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => self.sbc(&opcode.mode),
                0x38 => self.sec(),
                0xf8 => self.sed(),
                0x78 => self.sei(),
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.sta(&opcode.mode),
                0x86 | 0x96 | 0x8e => self.stx(&opcode.mode),
                0x84 | 0x94 | 0x8c => self.sty(&opcode.mode),
                0xaa => self.tax(),
                0xa8 => self.tay(),
                0xba => self.tsx(),
                0x8a => self.txa(),
                0x9a => self.txs(),
                0x98 => self.tya(),
                _ => todo!(""),
            }
            if before_program_counter == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }

    fn debug(&mut self, code: u8) {
        println!(
            "CPU,code:0x{:02x},ra:0x{:02x},rx:0x{:02x},ry:0x{:02x},st:0b{:08b},pc:0x{:04x},sp:0x{:02x}",
            code,
            self.register_a,
            self.register_x,
            self.register_y,
            self.status,
            self.program_counter,
            self.stack_pointer
        )
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

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_a = self.register_a & self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
    }

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

    fn asl_accumulator(&mut self) {
        if self.register_a >> 7 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.register_a = self.register_a * 2;
    }

    fn bcc(&mut self) {
        self.branch(self.status.contains(CpuFlags::CARRY));
    }

    fn bcs(&mut self) {
        self.branch(!self.status.contains(CpuFlags::CARRY));
    }

    fn beq(&mut self) {
        self.branch(self.status.contains(CpuFlags::ZERO));
    }

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

    fn bmi(&mut self) {
        self.branch(self.status.contains(CpuFlags::NEGATIVE));
    }

    fn bne(&mut self) {
        self.branch(!self.status.contains(CpuFlags::ZERO));
    }

    fn bpl(&mut self) {
        self.branch(!self.status.contains(CpuFlags::NEGATIVE));
    }

    // no brk function needed. 0x00 case only return

    fn bvc(&mut self) {
        self.branch(!self.status.contains(CpuFlags::OVERFLOW));
    }

    fn bvs(&mut self) {
        self.branch(self.status.contains(CpuFlags::OVERFLOW));
    }

    fn clc(&mut self) {
        self.status.remove(CpuFlags::CARRY);
    }

    fn cld(&mut self) {
        self.status.remove(CpuFlags::DECIMAL);
    }

    fn cli(&mut self) {
        self.status.remove(CpuFlags::INTERRUPT_DISABLE);
    }

    fn clv(&mut self) {
        self.status.remove(CpuFlags::OVERFLOW);
    }

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

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let new_data = data.wrapping_sub(1);
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        self.register_a = self.register_a ^ data;
        self.update_zero_and_negative_flags(self.register_a); // [TODO] maybe need.
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        let new_data = data.wrapping_add(1);
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read_u16(addr);
        self.program_counter = data;
    }

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

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_x = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.register_y = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_y);
    }

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

    fn lsr_accumulator(&mut self) {
        if self.register_a & 1 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.register_a = self.register_a >> 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn nop(&mut self) {
        // nothing to do
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        let data = self.mem_read(addr);
        self.register_a = data | self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    fn php(&mut self) {
        let mut data = self.status.clone();
        data.insert(CpuFlags::BREAK);
        data.insert(CpuFlags::RESERVED);
        self.stack_push(data.bits());
    }

    fn pla(&mut self) {
        self.register_a = self.stack_pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        self.status = CpuFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::RESERVED);
    }

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

    fn rti(&mut self) {
        self.status = CpuFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::RESERVED);

        self.program_counter = self.stack_pop_u16();
    }

    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

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

    fn sec(&mut self) {
        self.status.insert(CpuFlags::CARRY);
    }

    fn sed(&mut self) {
        self.status.insert(CpuFlags::DECIMAL);
    }

    fn sei(&mut self) {
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_adress(mode);
        self.mem_write(addr, self.register_y);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

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

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        let _ = self.stack_pointer.wrapping_sub(1);
    }

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
