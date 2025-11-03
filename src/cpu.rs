use crate::bus::Bus;
use crate::opcode::{self, OpCode};
use core::panic;
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
    NoneAddressing,
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
const STACK_TOP: u16 = 0x0100;
const INITIAL_STACK: u8 = 0xfd;

pub trait Memory {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}
pub struct CPU<'a> {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: Bus<'a>,

    pub extra_cycles: usize,
}

impl Memory for CPU<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        let d = self.bus.mem_read(addr);
        // println!("mem_read: addr:0x{:04x}, data:0x{:02x}", addr, d); // TODO
        return d;
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        // println!("mem_write: addr:0x{:04x}, data:0x{:02x}", addr, data); // TODO
        self.bus.mem_write(addr, data);
    }
}

impl<'a> CPU<'a> {
    pub fn new<'b>(bus: Bus<'b>) -> CPU<'b> {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(INITIAL_STATUS),
            program_counter: 0x8000,
            stack_pointer: INITIAL_STACK,
            bus: bus,
            extra_cycles: 0,
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        // self.mem_write_u16(0xfffc, 0x0600);
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0000 + i, program[i as usize]);
        }
        self.mem_write_u16(0xfffc, 0x8600);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = INITIAL_STACK;
        self.status = CpuFlags::from_bits_truncate(INITIAL_STATUS);

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    fn interrupt_nmi(&mut self) {
        // println!("interrupt nmi!!!!!!!!!!!!!!!!!!!"); // TODO
        self.stack_push_u16(self.program_counter);
        let mut flag = self.status.clone();
        flag.set(CpuFlags::BREAK, false);
        flag.set(CpuFlags::RESERVED, true);

        // println!("bits:{:04b}", flag);

        self.stack_push(flag.bits());
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
        // println!("new status:{:04b}", self.status);
        self.bus.tick(2);
        self.program_counter = self.mem_read_u16(0xfffa);
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        self.extra_cycles = 0;
        let ref opcodes: HashMap<u8, &'static OpCode> = *opcode::OPECODE_MAP;
        loop {
            if let Some(_nmi) = self.bus.poll_nmi_status() {
                self.interrupt_nmi();
            }
            callback(self);

            let code = self.mem_read(self.program_counter);
            // self.debug(code); // TODO
            // self.bus.show_ppu(); // TODO
            self.program_counter += 1;
            let before_program_counter = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));
            match code {
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => self.adc(&opcode.mode),
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => self.and(&opcode.mode),
                0x0a => self.asl_accumulator(),
                0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&opcode.mode);
                }
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
                0x4c => self.jmp_absolute(),
                0x6c => self.jmp(),
                0x20 => self.jsr(),
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.lda(&opcode.mode),
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.ldx(&opcode.mode),
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.ldy(&opcode.mode),
                0x4a => self.lsr_accumulator(),
                0x46 | 0x56 | 0x4e | 0x5e => {
                    self.lsr(&opcode.mode);
                }
                0xea => self.nop(),
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => self.ora(&opcode.mode),
                0x48 => self.pha(),
                0x08 => self.php(),
                0x68 => self.pla(),
                0x28 => self.plp(),
                0x2a => self.rol_accumulate(),
                0x26 | 0x36 | 0x2e | 0x3e => {
                    self.rol(&opcode.mode);
                }
                0x6a => self.ror_accumulator(),
                0x66 | 0x76 | 0x6e | 0x7e => {
                    self.ror(&opcode.mode);
                }
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
                0xa3 | 0xa7 | 0xaf | 0xb3 | 0xb7 | 0xbf => self.lax(&opcode.mode),
                0x83 | 0x87 | 0x8f | 0x97 => self.sax(&opcode.mode),
                0xeb => self.sbc(&opcode.mode),
                0xc3 | 0xc7 | 0xcf | 0xd3 | 0xd7 | 0xdb | 0xdf => self.dcp(&opcode.mode),
                0xe3 | 0xe7 | 0xef | 0xf3 | 0xf7 | 0xfb | 0xff => self.isc(&opcode.mode),
                0x03 | 0x07 | 0x17 | 0x0f | 0x1f | 0x1b | 0x13 => self.slo(&opcode.mode),
                0x27 | 0x37 | 0x2f | 0x3f | 0x3b | 0x23 | 0x33 => self.rla(&opcode.mode),
                0x47 | 0x57 | 0x4f | 0x5f | 0x5b | 0x43 | 0x53 => self.sre(&opcode.mode),
                0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => self.rra(&opcode.mode),
                0x04 | 0x44 | 0x64 | 0x0c | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 | 0x1a
                | 0x3a | 0x5a | 0x7a | 0xda | 0xfa | 0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 | 0x1c
                | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => self.nop(),
                _ => panic!("not arrowed operation code."),
            }

            self.bus
                .tick(opcode.cycle as usize + self.extra_cycles as usize);
            self.bus.print_cycle();

            if before_program_counter == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }

    #[allow(dead_code)]
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
        // println!("0x{:02x} st:0b{:08b}", code, self.status) // TODO
    }

    pub fn get_operand_address(&mut self, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Immediate => (self.program_counter, false),
            _ => self.get_absolute_address(mode, self.program_counter),
        }
    }

    fn is_page_crossed(&self, addr1: u16, addr2: u16) -> bool {
        let page_crossed = (addr1 & 0xFF00) != (addr2 & 0xFF);
        page_crossed
    }

    pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> (u16, bool) {
        match mode {
            // `page` is 256byte memory region.
            // for ex. 0page:0x0000 ~ 0x00ff, 1page:0x0100 ~ 0x01ff, ...
            // ZeroPage addressing uses only the first 256 bytes of memory, where the address is in the instruction
            AddressingMode::ZeroPage => (self.mem_read(addr) as u16, false),
            AddressingMode::Absolute => (self.mem_read_u16(addr), false),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_x) as u16;
                (addr, false)
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_y) as u16;
                (addr, false)
            }
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_x as u16);
                (addr, self.is_page_crossed(base, addr))
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_y as u16);
                (addr, self.is_page_crossed(base, addr))
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(addr);

                let ptr = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                let addr = (hi as u16) << 8 | lo as u16;
                (addr, false)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(addr);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | lo as u16;
                let deref = deref_base.wrapping_add(self.register_y as u16);
                (deref, self.is_page_crossed(deref_base, deref))
            }
            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.set_register_a_with_flags(data);

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn and(&mut self, mode: &AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode);
        self.register_a = self.register_a & self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn asl(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let carry = data >> 7;

        if carry == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        let new_data = data << 1;
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
        new_data
    }

    fn asl_accumulator(&mut self) {
        if self.register_a >> 7 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.register_a = self.register_a << 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn bcc(&mut self) {
        self.branch(!self.status.contains(CpuFlags::CARRY));
    }

    fn bcs(&mut self) {
        self.branch(self.status.contains(CpuFlags::CARRY));
    }

    fn beq(&mut self) {
        self.branch(self.status.contains(CpuFlags::ZERO));
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        // println!("addr:{}, val:{}", addr, data); // TODO for debug

        let and = self.register_a & data;
        if and == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }
        let flags = CpuFlags::NEGATIVE | CpuFlags::OVERFLOW;
        let v = data & flags.bits();
        self.status.remove(CpuFlags::NEGATIVE);
        self.status.remove(CpuFlags::OVERFLOW);
        self.status.insert(CpuFlags::from_bits_truncate(v));
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
        let (addr, page_crossed) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if self.register_a >= data {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(data));

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if self.register_x >= data {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_x.wrapping_sub(data));
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if self.register_y >= data {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(self.register_y.wrapping_sub(data));
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
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
        let (addr, page_crossed) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = self.register_a ^ data;
        self.update_zero_and_negative_flags(self.register_a); // [TODO] maybe need.

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
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

    fn jmp(&mut self) {
        // let (addr, _) = self.get_operand_address(mode);
        // let data = self.mem_read_u16(addr);
        // self.program_counter = data;

        let addr = self.mem_read_u16(self.program_counter);
        // let indirect_ref = self.mem_read_u16(mem_address);
        //6502 bug mode with with page boundary:
        //  if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
        // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
        // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

        let indirect_ref = if addr & 0x00FF == 0x00FF {
            let lo = self.mem_read(addr);
            let hi = self.mem_read(addr & 0xFF00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.mem_read_u16(addr)
        };

        self.program_counter = indirect_ref;
    }

    fn jmp_absolute(&mut self) {
        let address = self.mem_read_u16(self.program_counter);
        self.program_counter = address;
    }

    fn jsr(&mut self) {
        let pc = self.program_counter + 1;
        self.stack_push_u16(pc);
        let address = self.mem_read_u16(self.program_counter);
        self.program_counter = address;
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(value);

        // println!("addr:{:02x}, val:{}, st:0b{:08b}", addr, value, self.status); // TODO

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode);
        self.register_x = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_x);

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode);
        self.register_y = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_y);

        if page_crossed {
            self.extra_cycles += 1;
        }
    }

    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if data & 1 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        let new_data = data >> 1;
        self.mem_write(addr, new_data);
        self.update_zero_and_negative_flags(new_data);
        new_data
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
        let (addr, page_crossed) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data | self.register_a;
        self.update_zero_and_negative_flags(self.register_a);

        if page_crossed {
            self.extra_cycles += 1;
        }
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

    fn rol(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let carry = self.status.contains(CpuFlags::CARRY);
        let bit7 = data >> 7;
        if bit7 == 0 {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        data = data << 1;
        data = data | (carry as u8);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn rol_accumulate(&mut self) {
        let mut data = self.register_a;
        let carry = self.status.contains(CpuFlags::CARRY);
        let bit7 = data >> 7;
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

    fn ror(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
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
        data
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
        let (addr, page_crossed) = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        // let sub_val = ((data as i8).wrapping_neg().wrapping_sub(1)) as u8;
        // let overable_result = self.register_a as u16 + sub_val as u16 + carry;
        // [TODO] maybe ok.
        let target_val = (-(data as i16) - 1) as u8;
        self.set_register_a_with_flags(target_val);

        if page_crossed {
            self.extra_cycles += 1;
        }
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
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
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
        // println!("tsx:{}", self.register_x); // TODO
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn txs(&mut self) {
        // println!("txs:{}", self.register_x);
        self.stack_pointer = self.register_x;
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn lax(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data;
        self.register_x = data;
        self.update_zero_and_negative_flags(data);
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let data = self.register_a & self.register_x;
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, data);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        // https://www.masswerk.at/6502/6502_instruction_set.html#DCP
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let new_data = data.wrapping_sub(1);
        self.mem_write(addr, new_data);
        if new_data <= self.register_a {
            self.status.insert(CpuFlags::CARRY);
        }

        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(new_data));
    }

    // ISC or ISB
    // this operation has some names.
    fn isc(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        // let sub_val = ((data as i8).wrapping_neg().wrapping_sub(1)) as u8;
        // let overable_result = self.register_a as u16 + sub_val as u16 + carry;
        // [TODO] maybe ok.
        let target_val = (-(data as i16) - 2) as u8;
        self.set_register_a_with_flags(target_val);
        self.mem_write(addr, data.wrapping_add(1));
    }

    fn slo(&mut self, mode: &AddressingMode) {
        let data = self.asl(mode);
        let new_data = self.register_a | data;
        self.register_a = new_data;
        self.update_zero_and_negative_flags(new_data);
    }

    fn rla(&mut self, mode: &AddressingMode) {
        let data = self.rol(mode);
        let new_data = self.register_a & data;
        self.register_a = new_data;
        self.update_zero_and_negative_flags(new_data);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        let data = self.lsr(mode);
        let new_data = self.register_a ^ data;
        self.register_a = new_data;
        self.update_zero_and_negative_flags(new_data);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        let data = self.ror(mode);
        self.set_register_a_with_flags(data);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            self.extra_cycles += 1;

            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            if self.program_counter.wrapping_add(1) & 0xFF00 != jump_addr & 0xFF00 {
                self.extra_cycles += 1;
            }

            self.program_counter = jump_addr;
        }

        // let jump = self.mem_read(self.program_counter) as i8;
        // let old_pc = self.program_counter.wrapping_add(1);
        // let new_pc = old_pc.wrapping_add(jump as u16);

        // if condition {
        //     println!(""); // TODO
        //     self.program_counter = new_pc;
        //     self.bus.tick(1);
        //     if self.is_page_crossed(old_pc, new_pc) {
        //         println!("page crossed"); // TODO
        //         self.bus.tick(1); // FIXME
        //     }
        // }

        // println!(
        //     "c:{}, jmp:0x{:04x}, old:0x{:04x}, new:0x{:04x}",
        //     condition, jump, old_pc, new_pc
        // );
        // println!("old:0x{:04x}, new:0x{:04x}", old_pc, new_pc); // TODO
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }
        if result >> 7 == 1 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    fn set_register_a_with_flags(&mut self, set_data: u8) {
        let carry = if self.status.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        };

        let overable_result = self.register_a as i16 + set_data as i16 + carry as i16;
        if overable_result > 0xff {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        let result = overable_result as u8;

        if (result ^ self.register_a) & (result ^ set_data) & 0x80 == 0 {
            self.status.remove(CpuFlags::OVERFLOW);
        } else {
            self.status.insert(CpuFlags::OVERFLOW);
        }

        self.register_a = result;
        self.update_zero_and_negative_flags(result);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let val = self.mem_read((STACK_TOP as u16) + self.stack_pointer as u16);
        // println!(
        //     "pop: pointer:{}, val:{}, adddr:{}",
        //     self.stack_pointer,
        //     val,
        //     (STACK_TOP as u16) + self.stack_pointer as u16
        // );
        return val;
    }

    fn stack_push(&mut self, data: u8) {
        let addr = (STACK_TOP as u16) + self.stack_pointer as u16;
        self.mem_write(addr, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        // println!(
        //     "push: pointer:{}, addr:{}, val:{}",
        //     self.stack_pointer, addr, data
        // ); // TODO
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
    use crate::{cartoridge::Rom, joypad::Joypad, ppu::NesPPU};

    use super::*;

    // NESヘッダー
    const TEST_HEADER: [u8; 16] = [
        0x4E, 0x45, 0x53, 0x1A, // NES^Z
        0x02, // PRG ROMサイズ (16KB単位)
        0x01, // CHR ROMサイズ (8KB単位)
        0x31, // フラグ6
        0x00, // フラグ7
        0x00, 0x00, 0x00, 0x00, // 予約領域
        0x00, 0x00, 0x00, 0x00, // 予約領域
    ];

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let testdata = [0xa9, 0x05, 0x00];

        let mut rom_data = Vec::new();
        rom_data.extend_from_slice(&TEST_HEADER);
        rom_data.extend_from_slice(&testdata);
        rom_data.resize(rom_data.len() + 2 * 16 * 1024, 0);
        rom_data.extend_from_slice(&[2; 1 * 8 * 1024]);

        let rom = Rom::new(&rom_data).unwrap();
        let bus = Bus::new(rom, |_: &NesPPU, _: &mut Joypad| {});
        let mut cpu = CPU::new(bus);
        cpu.run();

        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let testdata = [0xa9, 0x00, 0x00];

        let mut rom_data = Vec::new();
        rom_data.extend_from_slice(&TEST_HEADER);
        rom_data.extend_from_slice(&testdata);
        rom_data.resize(rom_data.len() + 2 * 16 * 1024, 0);
        rom_data.extend_from_slice(&[2; 1 * 8 * 1024]);

        let rom = Rom::new(&rom_data).unwrap();
        let bus = Bus::new(rom, |_: &NesPPU, _: &mut Joypad| {});
        let mut cpu = CPU::new(bus);
        cpu.run();

        assert!(cpu.status.contains(CpuFlags::ZERO))
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let testdata = [0xa9, 0x0a, 0xaa, 0x00];

        let mut rom_data = Vec::new();
        rom_data.extend_from_slice(&TEST_HEADER);
        rom_data.extend_from_slice(&testdata);
        rom_data.resize(rom_data.len() + 2 * 16 * 1024, 0);
        rom_data.extend_from_slice(&[2; 1 * 8 * 1024]);

        let rom = Rom::new(&rom_data).unwrap();
        let bus = Bus::new(rom, |_: &NesPPU, _: &mut Joypad| {});
        let mut cpu = CPU::new(bus);
        cpu.run();

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_inx_overflow() {
        let testdata = [0xa9, 0xff, 0xaa, 0xe8, 0xe8];

        let mut rom_data = Vec::new();
        rom_data.extend_from_slice(&TEST_HEADER);
        rom_data.extend_from_slice(&testdata);
        rom_data.resize(rom_data.len() + 2 * 16 * 1024, 0);
        rom_data.extend_from_slice(&[2; 1 * 8 * 1024]);

        let rom = Rom::new(&rom_data).unwrap();
        let bus = Bus::new(rom, |_: &NesPPU, _: &mut Joypad| {});
        let mut cpu = CPU::new(bus);
        cpu.run();

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_5_ops_working_togather() {
        let testdata = [0xa9, 0xc0, 0xaa, 0xe8, 0x00];

        let mut rom_data = Vec::new();
        rom_data.extend_from_slice(&TEST_HEADER);
        rom_data.extend_from_slice(&testdata);
        rom_data.resize(rom_data.len() + 2 * 16 * 1024, 0);
        rom_data.extend_from_slice(&[2; 1 * 8 * 1024]);

        let rom = Rom::new(&rom_data).unwrap();
        let bus = Bus::new(rom, |_: &NesPPU, _: &mut Joypad| {});
        let mut cpu = CPU::new(bus);
        cpu.run();

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_lda_from_memory() {
        let testdata = [0xa5, 0x10, 0x00];

        let mut rom_data = Vec::new();
        rom_data.extend_from_slice(&TEST_HEADER);
        rom_data.extend_from_slice(&testdata);
        rom_data.resize(rom_data.len() + 2 * 16 * 1024, 0);
        rom_data.extend_from_slice(&[2; 1 * 8 * 1024]);

        let rom = Rom::new(&rom_data).unwrap();
        let bus = Bus::new(rom, |_: &NesPPU, _: &mut Joypad| {});
        let mut cpu = CPU::new(bus);

        cpu.mem_write(0x10, 0x55); // set test data
        cpu.run();

        assert_eq!(cpu.register_a, 0x55)
    }
}
