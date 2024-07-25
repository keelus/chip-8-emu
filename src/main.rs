//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

mod core;
use core::beep;
use core::{cpu::Cpu, screen};
use std::fs;

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::AudioSubsystem;

// Sample SquareWave struct code from SDL2's example
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

// Sample AudioCallback impl code from SDL2's example
impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

struct BeepHandler {
    device: Option<AudioDevice<SquareWave>>,
    desired_spec: AudioSpecDesired,
    audio_subsystem: AudioSubsystem,
}

impl beep::BeepHandler for BeepHandler {
    fn start(&mut self) {
        if self.device.is_none() {
            let new_device = self
                .audio_subsystem
                .open_playback(None, &self.desired_spec, |spec| SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                })
                .unwrap();

            new_device.resume();
            self.device = Some(new_device);
        }
    }

    fn stop(&mut self) {
        if self.device.is_some() {
            let cur_device = self.device.take().unwrap();
            cur_device.close_and_get_callback();
            self.device = None;
        }
    }
}

const PROGRAM_BEGIN: u16 = 0x0200;

const WINDOW_SCALE: u32 = 10;
const WINDOW_WIDTH: u32 = (screen::WIDTH as u32) * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = (screen::HEIGHT as u32) * WINDOW_SCALE;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

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

    let rom_name = "1-chip8-logo";
    let rom = fs::read(format!("./roms/{}.ch8", rom_name)).unwrap();
    let mut cpu = Cpu::new(rom, PROGRAM_BEGIN);

    cpu.add_beep_handler(Box::new(BeepHandler {
        device: None,
        audio_subsystem,
        desired_spec,
    }));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keycode {
                        match key {
                            Keycode::Num1 => cpu.keypad.set_key(1, false),
                            Keycode::Num2 => cpu.keypad.set_key(2, false),
                            Keycode::Num3 => cpu.keypad.set_key(3, false),
                            Keycode::Num4 => cpu.keypad.set_key(0xC, false),

                            Keycode::Q => cpu.keypad.set_key(4, false),
                            Keycode::W => cpu.keypad.set_key(5, false),
                            Keycode::E => cpu.keypad.set_key(6, false),
                            Keycode::R => cpu.keypad.set_key(0xD, false),

                            Keycode::A => cpu.keypad.set_key(7, false),
                            Keycode::S => cpu.keypad.set_key(8, false),
                            Keycode::D => cpu.keypad.set_key(9, false),
                            Keycode::F => cpu.keypad.set_key(0xE, false),

                            Keycode::Z => cpu.keypad.set_key(0xA, false),
                            Keycode::X => cpu.keypad.set_key(0, false),
                            Keycode::C => cpu.keypad.set_key(0xB, false),
                            Keycode::V => cpu.keypad.set_key(0xF, false),
                            _ => {}
                        }
                    }
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = keycode {
                        match key {
                            Keycode::Num1 => cpu.keypad.set_key(1, true),
                            Keycode::Num2 => cpu.keypad.set_key(2, true),
                            Keycode::Num3 => cpu.keypad.set_key(3, true),
                            Keycode::Num4 => cpu.keypad.set_key(0xC, true),

                            Keycode::Q => cpu.keypad.set_key(4, true),
                            Keycode::W => cpu.keypad.set_key(5, true),
                            Keycode::E => cpu.keypad.set_key(6, true),
                            Keycode::R => cpu.keypad.set_key(0xD, true),

                            Keycode::A => cpu.keypad.set_key(7, true),
                            Keycode::S => cpu.keypad.set_key(8, true),
                            Keycode::D => cpu.keypad.set_key(9, true),
                            Keycode::F => cpu.keypad.set_key(0xE, true),

                            Keycode::Z => cpu.keypad.set_key(0xA, true),
                            Keycode::X => cpu.keypad.set_key(0, true),
                            Keycode::C => cpu.keypad.set_key(0xB, true),
                            Keycode::V => cpu.keypad.set_key(0xF, true),
                            _ => {}
                        }
                    }
                }
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

        cpu.tick();
    }
}
