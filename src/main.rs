//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

mod core;
use core::{cpu::Cpu, screen};
use std::borrow::BorrowMut;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

const PROGRAM_BEGIN: u16 = 0x0200;

const WINDOW_SCALE: u32 = 10;
const WINDOW_WIDTH: u32 = (screen::WIDTH as u32) * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = (screen::HEIGHT as u32) * WINDOW_SCALE;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Snake ROM", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas
        .set_scale(WINDOW_SCALE as f32, WINDOW_SCALE as f32)
        .unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(
            PixelFormatEnum::RGB24,
            screen::WIDTH as u32,
            screen::HEIGHT as u32,
        )
        .unwrap();

    let mut buffer = [0 as u8; (screen::WIDTH * screen::HEIGHT * 3) as usize];

    let mut cpu = Cpu::new(vec![0x80, 0x32], PROGRAM_BEGIN);

    // Example pixel top left
    let pix = cpu.screen.0[0].borrow_mut();
    *pix |= 0x1 << 63;

    // Example pixel top right
    let pix = cpu.screen.0[0].borrow_mut();
    *pix |= 1;

    // Example pixel bottom left
    let pix = cpu.screen.0[screen::HEIGHT - 1].borrow_mut();
    *pix |= 0x1 << 63;

    // Example pixel bottom right
    let pix = cpu.screen.0[screen::HEIGHT - 1].borrow_mut();
    *pix |= 1;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let mut buff_idx = 0;
        for row in cpu.screen.0 {
            for col in 0x0..screen::WIDTH {
                let mask: u64 = 0x1 << (63 - col);
                let pixel_on = (row & mask) != 0;

                if pixel_on {
                    buffer[buff_idx] = 0xFF;
                    buffer[buff_idx + 1] = 0xFF;
                    buffer[buff_idx + 2] = 0xFF;
                } else {
                    buffer[buff_idx] = 0x0;
                    buffer[buff_idx + 1] = 0x0;
                    buffer[buff_idx + 2] = 0x0;
                }

                buff_idx += 3;
            }
        }

        texture.update(None, &buffer, screen::WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        //cpu.tick();
    }
}
