use crate::cartoridge::Rom;
use crate::cpu::Memory;
use crate::ppu::{NesPPU, PPU};

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,
    cycle: usize,
    gameloop_callback: Box<dyn FnMut(&NesPPU) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);
        Bus {
            cpu_vram: [0; 0x0800], // 2048
            prg_rom: rom.prg_rom,
            ppu: ppu,
            cycle: 0,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    pub fn show_ppu_status(&self) {
        self.ppu.show_cycle_and_scanline();
    }

    pub fn print_cycle(&self) {
        // println!("bus cycle: {}", self.cycle);
    }

    pub fn tick(&mut self, cycles: u8) {
        // println!("before: {}", self.cycle);
        self.cycle += cycles as usize;
        let new_frame = self.ppu.tick(cycles * 3);
        if new_frame {
            (self.gameloop_callback)(&self.ppu);
        }
        // println!("after: {}", self.cycle);
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
// const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRROR_END: u16 = 0x3FFF;

impl Memory for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr)
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRROR_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }
            0x4000..=0x4015 => {
                //ignore APU
                0
            }

            0x4016 => {
                // ignore joypad 1;
                0
            }

            0x4017 => {
                // ignore joypad 2
                0
            }
            0x8000..=0xFFFF => self.read_prg_rom(addr),

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }
    fn mem_write(&mut self, addr: u16, data: u8) {
        // println!("mem_write addr:0x{:04x}, data:0x{:02x}", addr, data); // TODO
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2001 => {
                self.ppu.write_to_mask(data);
            }
            0x2002 => {
                panic!("read only PPU adress {:x}", addr);
            }
            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }
            0x2004 => {
                self.ppu.write_to_oam_data(data);
            }
            0x2005 => {
                self.ppu.write_to_scroll(data);
            }
            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            0x2008..=PPU_REGISTERS_MIRROR_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            0x4000..=0x4013 => {
                // TODO APU
            }
            0x4014 => {
                // TODO OAMDMA
            }
            0x4015 => {
                // TODO SND_CHN
            }
            0x4016 => {
                // TODO joypad1
            }
            0x4017 => {
                // TODOjoypad2
            }
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => println!("Ignoring mem write-access at {}", addr),
        }
    }
}
