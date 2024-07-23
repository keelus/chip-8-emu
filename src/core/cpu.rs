//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use super::{keypad::Keypad, memory::Memory, registers::Registers, screen::Screen};

pub struct Cpu {
    registers: Registers,
    memory: Memory,
    screen: Screen,
    keypad: Keypad,
}

impl Cpu {
    pub fn new(program: Vec<u8>, program_begin: u16) -> Cpu {
        Cpu {
            registers: Registers::new(program_begin),
            memory: Memory::new(program, program_begin),
            screen: Screen::new(),
            keypad: Keypad::new(),
        }
    }
}
