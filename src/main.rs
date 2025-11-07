use std::collections::HashMap;
use std::env;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use crate::bus::Bus;
use crate::cartoridge::Rom;
use crate::frame::Frame;
use crate::joypad::Joypad;
use crate::ppu::NesPPU;
use cpu::CPU;

pub mod bus;
pub mod cartoridge;
pub mod control;
pub mod cpu;
pub mod frame;
pub mod joypad;
pub mod mask;
pub mod opcode;
pub mod palette;
pub mod ppu;
pub mod render;
pub mod scroll;
pub mod status;
pub mod trace;

fn main() {
    const LOGICAL_WIDTH: u32 = 256;
    const LOGICAL_HEIGHT: u32 = 240;
    const WINDOW_SCALE: u32 = 3;

    let args: Vec<String> = env::args().collect();
    let rom_path = if args.len() > 2 {
        &args[1]
    } else {
        "./test/sample/nestest.nes"
    };

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "NES Emulator",
            (LOGICAL_WIDTH * WINDOW_SCALE) as u32,
            (LOGICAL_HEIGHT * WINDOW_SCALE) as u32,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas
        .set_scale(WINDOW_SCALE as f32, WINDOW_SCALE as f32)
        .unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, LOGICAL_WIDTH, LOGICAL_HEIGHT)
        .unwrap();

    let bytes = std::fs::read(rom_path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let mut frame = Frame::new();

    let bus = Bus::new(rom, move |ppu: &NesPPU, joypad: &mut Joypad| {
        render::render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, false);
                    }
                }
                _ => { /* nop */ }
            }
        }
    });

    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.run();
}
