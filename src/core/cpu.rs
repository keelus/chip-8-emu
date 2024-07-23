//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use std::ops::Shr;

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

        match instruction.parts() {
            (0, 0, 0xE, 0) => panic!("CLS not implemented."),
            (0, 0, 0xE, 0xE) => panic!("RET not implemented."),
            (0, _, _, _) => panic!("SYS not implemented."),
            (1, _, _, _) => panic!("JP not implemented."),
            (2, _, _, _) => panic!("CALL not implemented."),
            (3, _, _, _) => panic!("SE|b not implemented."),
            (4, _, _, _) => panic!("SNE|b not implemented."),
            (5, _, _, 0) => panic!("SE|vy not implemented."),
            (6, _, _, _) => {
                // LD
                let x = instruction.x();
                let kk = instruction.kk();
                self.registers.v[x as usize] = kk;
            }
            (7, _, _, _) => {
                // ADD (no carry)
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                let vx = vx.wrapping_add(x);
                self.registers.v[x as usize] = vx;
            }
            (8, _, _, 0) => {
                // LD
                let x = instruction.x();
                let y = instruction.y();
                self.registers.v[x as usize] = self.registers.v[y as usize];
            }
            (8, _, _, 1) => {
                // OR
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                self.registers.v[x as usize] = vx | vy;
            }
            (8, _, _, 2) => {
                // AND
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                self.registers.v[x as usize] = vx & vy;
            }
            (8, _, _, 3) => {
                // XOR
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                self.registers.v[x as usize] = vx ^ vy;
            }
            (8, _, _, 4) => {
                // ADD
                let x = instruction.x();
                let y = instruction.x();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (vx, overflows) = vx.overflowing_add(vy);
                self.registers.v[x as usize] = vx;
                self.registers.v[0x0F] = if overflows { 1 } else { 0 };
            }
            (8, _, _, 5) => {
                // SUB
                let x = instruction.x();
                let y = instruction.x();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (vx, overflows) = vx.overflowing_sub(vy);
                self.registers.v[x as usize] = vx;
                self.registers.v[0x0F] = if overflows { 1 } else { 0 };
            }
            (8, _, _, 6) => {
                // SHR
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                self.registers.v[0x0F] = if vx & 0x1 != 0 { 1 } else { 0 };
                let vx = vx >> 1;
                self.registers.v[x as usize] = vx;
            }
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

        self.registers.pc += 2
    }
}
