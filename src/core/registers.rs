//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

pub struct Registers {
    pub v: [u8; 16], // General purpose
    pub i: u16,      // Memory address oriented [12bit]
    pub timers: [u8; 2],
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
}

impl Registers {
    pub fn new(pc_begin: u16) -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            timers: [0; 2],
            pc: pc_begin,
            sp: 0,
            stack: [0; 16],
        }
    }
}
