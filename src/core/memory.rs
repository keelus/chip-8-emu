//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

const MEMORY_SIZE: usize = 4096;

pub struct Memory([u8; MEMORY_SIZE]);

impl Memory {
    pub fn new() -> Memory {
        Memory {
            0: [0; MEMORY_SIZE],
        }
    }
}
