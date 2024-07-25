//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use std::time::Instant;

pub struct Timer {
    last_write: Instant,
    write_data: u8,
}

pub const TIMER_HZ: f64 = 60.0;

impl Timer {
    pub fn new() -> Timer {
        Timer {
            last_write: Instant::now(),
            write_data: 0,
        }
    }

    pub fn write(&mut self, data: u8) {
        self.write_data = data;
        self.last_write = Instant::now()
    }

    pub fn read(&mut self) -> u8 {
        let now = Instant::now();
        let diff = now.duration_since(self.last_write);

        let diff_s = diff.as_secs_f64();
        let value = self.write_data as f64 - TIMER_HZ * diff_s;
        let value = value.round() as u8;

        value
    }
}

pub const DELAY_TIMER: usize = 0;
pub const SOUND_TIMER: usize = 1;

pub struct Registers {
    pub v: [u8; 16], // General purpose
    pub i: u16,      // Memory address oriented [12bit]
    //pub timers: [u8; 2],
    pub timers: [Timer; 2],
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
}

impl Registers {
    pub fn new(pc_begin: u16) -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            timers: [Timer::new(), Timer::new()],
            pc: pc_begin,
            sp: 0,
            stack: [0; 16],
        }
    }
}
