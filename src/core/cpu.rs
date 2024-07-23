//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use core::panicking::panic;

use super::{keypad::Keypad, memory::Memory, registers::Registers, screen::Screen};

pub struct Cpu {
    pub registers: Registers,
    pub memory: Memory,
    pub screen: Screen,
    pub keypad: Keypad,
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

    pub fn tick(&mut self) {
        let instruction = self.memory.read_instruction(self.registers.pc);

        match instruction {
            (0, 0, 0xE, 0) => panic!("CLS not implemented."),
            (0, 0, 0xE, 0xE) => panic!("RET not implemented."),
            (0, _, _, _) => panic!("SYS not implemented."),
            (1, _, _, _) => panic!("JP not implemented."),
            (2, _, _, _) => panic!("CALL not implemented."),
            (3, _, _, _) => panic!("SE|b not implemented."),
            (4, _, _, _) => panic!("SNE|b not implemented."),
            (5, _, _, 0) => panic!("SE|vy not implemented."),
            (6, _, _, _) => panic!("LD not implemented."),
            (7, _, _, _) => panic!("ADD not implemented."),
            (8, _, _, 0) => panic!("LD not implemented."),
            (8, _, _, 1) => panic!("OR not implemented."),
            (8, _, _, 2) => panic!("AND not implemented."),
            (8, _, _, 3) => panic!("XOR not implemented."),
            (8, _, _, 4) => panic!("ADD not implemented."),
            (8, _, _, 5) => panic!("SUB not implemented."),
            (8, _, _, 6) => panic!("SHR not implemented."),
            (8, _, _, 7) => panic!("SUBN not implemented."),
            (8, _, _, 0xE) => panic!("SHL not implemented."), // E ?
            (9, _, _, 0) => panic!("SNE|vy not implemented."),
            (0xA, _, _, _) => panic!("LD not implemented."),
            (0xB, _, _, _) => panic!("JPv0 not implemented."),
            (0xC, _, _, _) => panic!("RND not implemented."),
            (0xD, _, _, _) => panic!("DRW not implemented."),
            (0xE, _, 9, 0xE) => panic!("SKP not implemented."),
            (0xE, _, 0xA, 1) => panic!("SKNP not implemented."),
            (0xF, _, 0, 7) => panic!("LD not implemented."),
            (0xF, _, 0, 0xA) => panic!("LD not implemented."),
            (0xF, _, 1, 5) => panic!("LD not implemented."),
            (0xF, _, 1, 8) => panic!("LD not implemented."),
            (0xF, _, 1, 0xE) => panic!("ADD not implemented."),
            (0xF, _, 2, 9) => panic!("LD not implemented."),
            (0xF, _, 3, 3) => panic!("LD not implemented."),
            (0xF, _, 5, 5) => panic!("LD not implemented."),
            (0xF, _, 6, 5) => panic!("LD not implemented."),
            _ => panic!("Unknown instruction."),
        }
    }
}
