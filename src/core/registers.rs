//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

pub struct Registers {
    v: [u8; 16], // General purpose
    i: u16,      // Memory address oriented [12bit]
    timers: [u8; 2],
    pc: u16,
    sp: u8,
    stack: [u16; 16],
}
