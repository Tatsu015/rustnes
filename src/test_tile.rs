use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use crate::cartoridge::Rom;
use crate::frame::Frame;

pub mod bus;
pub mod cartoridge;
pub mod control;
pub mod cpu;
pub mod frame;
pub mod mask;
pub mod opcode;
pub mod palette;
pub mod ppu;
pub mod scroll;
pub mod status;
pub mod trace;

fn main() {
    const LOGICAL_WIDTH: u32 = 256;
    const LOGICAL_HEIGHT: u32 = 240;
    const WINDOW_SCALE: u32 = 3;
    const BYTES_PER_PIXEL: u32 = 3;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Snake game",
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

    let bytes = std::fs::read("./test/sample/sample1.nes").unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let tile_frame = show_tile_brank(&rom.chr_rom, 1);
    texture
        .update(
            None,
            &tile_frame.data,
            (LOGICAL_WIDTH * BYTES_PER_PIXEL) as usize,
        )
        .unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => { /* nop */ }
            }
        }
    }
}

// fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize) -> Frame {
//     assert!(bank <= 1);

//     let mut frame = Frame::new();
//     let bank = (bank * 0x1000) as usize;

//     let tile_start_pos = bank + tile_n * 16;
//     let tile = &chr_rom[tile_start_pos..=tile_start_pos + 15];

//     for y in 0..7 {
//         let mut upper = tile[y];
//         let mut lower = tile[y + 8];

//         for x in (0..=7).rev() {
//             let value = (1 & upper) << 1 | (1 & lower);
//             upper = upper >> 1;
//             lower = lower >> 1;
//             let rgb = match value {
//                 0 => SYSTEM_PALLETE[0x01],
//                 1 => SYSTEM_PALLETE[0x23],
//                 2 => SYSTEM_PALLETE[0x27],
//                 3 => SYSTEM_PALLETE[0x30],
//                 _ => panic!("can't be"),
//             };
//             frame.set_pixcel(x, y, rgb)
//         }
//     }

//     frame
// }

fn show_tile_brank(chr_rom: &Vec<u8>, bank: usize) -> Frame {
    assert!(bank <= 11);

    let mut frame = Frame::new();
    let bank = (bank * 0x1000) as usize;
    let mut tile_y = 0;
    let mut tile_x = 0;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile_start_pos = bank + tile_n * 16;
        let tile = &chr_rom[tile_start_pos..=tile_start_pos + 15];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[0x01],
                    1 => palette::SYSTEM_PALLETE[0x23],
                    2 => palette::SYSTEM_PALLETE[0x27],
                    3 => palette::SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixcel(tile_x + x, tile_y + y, rgb)
            }
        }
        tile_x += 10;
    }

    frame
}
