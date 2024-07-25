//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use std::ops::Shr;

use rand::Rng;

use crate::core::{memory::HEX_SPRITES_WIDTH, screen};

use super::{
    keypad::Keypad,
    memory::{Memory, HEX_SPRITES_HEIGHT, HEX_SPRITES_START_MEM},
    registers::Registers,
    screen::Screen,
};

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
            (0, 0, 0xE, 0) => {
                // CLS - 00e0
                self.screen.clear();
            }
            (0, 0, 0xE, 0xE) => {
                // RET - 00ee
                let (mut sp, overflows) = self.registers.sp.overflowing_sub(1);
                if overflows {
                    sp = 0xF;
                }
                let pc = self.registers.stack[sp as usize];

                self.registers.sp = sp;
                self.registers.pc = pc;
            }
            (0, _, _, _) => panic!("SYS not implemented."),
            (1, _, _, _) => {
                // JP - 1nnn
                let nnn = instruction.nnn();
                self.registers.pc = nnn.wrapping_sub(2); // So tick()'s pc+=2 at the end doesn't affect
            }
            (2, _, _, _) => {
                // CALL - 2nnn
                let nnn = instruction.nnn();

                let sp = self.registers.sp;
                self.registers.stack[sp as usize] = self.registers.pc;

                let mut sp = sp + 1;
                if sp > 0xF {
                    sp = 0x0;
                }

                self.registers.sp = sp;
                self.registers.pc = nnn.wrapping_sub(2);
            }
            (3, _, _, _) => {
                // SE - 3xkk
                let kk = instruction.kk();
                let x = instruction.x();
                let vx = self.registers.v[x as usize];

                if vx == kk {
                    self.registers.pc += 2
                }
            }
            (4, _, _, _) => {
                // SNE - 4xkk
                let kk = instruction.kk();
                let x = instruction.x();
                let vx = self.registers.v[x as usize];

                if vx != kk {
                    self.registers.pc += 2
                }
            }
            (5, _, _, 0) => {
                // SE - 5xy0
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];

                if vx == vy {
                    self.registers.pc += 2
                }
            }
            (6, _, _, _) => {
                // LD - 6xkk
                let x = instruction.x();
                let kk = instruction.kk();
                self.registers.v[x as usize] = kk;
            }
            (7, _, _, _) => {
                // ADD (no carry) - 7xkk
                let x = instruction.x();
                let kk = instruction.kk();
                let vx = self.registers.v[x as usize];
                let vx = vx.wrapping_add(kk);
                self.registers.v[x as usize] = vx;
            }
            (8, _, _, 0) => {
                // LD - 8xy0
                let x = instruction.x();
                let y = instruction.y();
                self.registers.v[x as usize] = self.registers.v[y as usize];
            }
            (8, _, _, 1) => {
                // OR - 8xy1
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                self.registers.v[x as usize] = vx | vy;
            }
            (8, _, _, 2) => {
                // AND - 8xy2
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                self.registers.v[x as usize] = vx & vy;
            }
            (8, _, _, 3) => {
                // XOR - 8xy3
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                self.registers.v[x as usize] = vx ^ vy;
            }
            (8, _, _, 4) => {
                // ADD - 8xy4
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (vx, overflows) = vx.overflowing_add(vy);
                self.registers.v[x as usize] = vx;
                self.registers.v[0x0F] = if overflows { 1 } else { 0 };
            }
            (8, _, _, 5) => {
                // SUB - 8xy5
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (vx, overflows) = vx.overflowing_sub(vy);
                self.registers.v[x as usize] = vx;
                self.registers.v[0x0F] = if overflows { 1 } else { 0 };
            }
            (8, _, _, 6) => {
                // SHR - 8xy6
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                self.registers.v[0x0F] = if vx & 0x1 != 0 { 1 } else { 0 };
                let vx = vx >> 1;
                self.registers.v[x as usize] = vx;
            }
            (8, _, _, 7) => {
                // SUBN - 8xy7
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (vx, overflows) = vy.overflowing_sub(vx);
                self.registers.v[x as usize] = vx;
                self.registers.v[0x0F] = if overflows { 1 } else { 0 };
            }
            (8, _, _, 0xE) => {
                // SHL - 8xye
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                self.registers.v[0x0F] = if vx & 0x80 != 0 { 1 } else { 0 };
                let vx = vx << 1;
                self.registers.v[x as usize] = vx;
            }
            (9, _, _, 0) => {
                // SNE - 9xy0
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];

                if vx != vy {
                    self.registers.pc += 2
                }
            }
            (0xA, _, _, _) => {
                // LD - annn
                let nnn = instruction.nnn();
                self.registers.i = nnn;
            }
            (0xB, _, _, _) => {
                // JP - bnnn
                let nnn = instruction.nnn();
                let v0 = self.registers.v[0] as u16;
                let pc = nnn.wrapping_add(v0);
                self.registers.pc = pc.wrapping_sub(2);
            }
            (0xC, _, _, _) => {
                // RND - cxkk
                let x = instruction.x();
                let kk = instruction.kk();
                let rnd: u8 = rand::thread_rng().gen_range(0..=255);
                let rnd = rnd & kk;
                self.registers.v[x as usize] = rnd;
            }
            (0xD, _, _, _) => {
                // DRW - dxyn
                let i = self.registers.i;
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let n = instruction.n() as u16;

                let mut x = vx;
                let mut y = vy;

                for idx in 0..n {
                    let addr = HEX_SPRITES_START_MEM + i + idx;
                    let data = (self.memory.read(addr) as u64) << 56;
                    let collision = (self.screen.0[y as usize] & data.rotate_right(x as u32)) != 0;
                    //println!("Collision: {:#066x} ({})", collision, collision);
                    self.screen.0[y as usize] ^= data.rotate_right(x as u32);

                    self.registers.v[0xF] = if collision { 1 } else { 0 };

                    y += 1;
                    if y >= screen::HEIGHT as u8 {
                        y = 0;
                    }
                }
            }
            (0xE, _, 9, 0xE) => {
                // SKP - ex9e
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                let is_down = self.keypad.get_key_state(vx);

                if is_down {
                    self.registers.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                // SKNP - exa1
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                let is_down = self.keypad.get_key_state(vx);

                if !is_down {
                    self.registers.pc += 2;
                }
            }
            (0xF, _, 0, 7) => panic!("LD not implemented."),
            (0xF, _, 0, 0xA) => {
                // LD - fx0a
                let key = self.keypad.get_released_key();

                if let Some(key) = key {
                    let x = instruction.x();
                    self.registers.v[x as usize] = key;
                } else {
                    self.registers.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => panic!("LD not implemented."),
            (0xF, _, 1, 8) => panic!("LD not implemented."),
            (0xF, _, 1, 0xE) => {
                // ADD (no carry) - fx1e
                let x = instruction.x();
                let vx = self.registers.v[x as usize] as u16;
                let i = self.registers.i;
                let i = i.wrapping_add(vx);
                self.registers.i = i;
            }
            (0xF, _, 2, 9) => {
                // LD - fx29
                let x = instruction.x() as u16;
                let addr = HEX_SPRITES_START_MEM.wrapping_add(x * HEX_SPRITES_HEIGHT as u16);
                self.registers.i = addr;
            }
            (0xF, _, 3, 3) => {
                // LD - fx33
                let x = instruction.x();
                let vx = self.registers.v[x as usize];
                let i = self.registers.i;

                let hundreds = vx / 100;
                let tens = (vx / 10) % 10;
                let ones = vx % 10;

                self.memory.write(i, hundreds);
                self.memory.write(i + 1, tens);
                self.memory.write(i + 2, ones);
            }
            (0xF, _, 5, 5) => {
                // LD [x inclusive. TODO: Check if I is updated] - fx55
                let x = instruction.x();
                let mut addr = self.registers.i;
                for idx in 0..=x {
                    let v = self.registers.v[idx as usize];
                    self.memory.write(addr, v);
                    addr += 1;
                }
            }
            (0xF, _, 6, 5) => {
                // LD [x inclusive. TODO: Check if I is updated] - fx65
                let x = instruction.x();
                let mut addr = self.registers.i;
                for idx in 0..=x {
                    let v = self.memory.read(addr);
                    self.registers.v[idx as usize] = v;
                    addr += 1;
                }
            }
            _ => panic!("Unknown instruction."),
        }

        self.registers.pc += 2
    }
}

#[cfg(test)]
mod instruction_tests {
    use crate::core::cpu::Cpu;

    // CLS
    // SYS

    #[test]
    fn test_ret_00ee() {
        let mut cpu = Cpu::new(vec![0x00, 0xEE], 0x0200);
        cpu.registers.sp = 0;
        cpu.registers.stack[0xF] = 0x0300;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x302);
        assert_eq!(cpu.registers.sp, 0xF);
    }
    #[test]
    fn test_ret_00ee_full() {
        let mut cpu = Cpu::new(vec![0x00, 0xEE], 0x0200);
        cpu.registers.sp = 0x1;
        cpu.registers.stack[0x0] = 0x0300;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x302);
        assert_eq!(cpu.registers.sp, 0x0);
    }

    #[test]
    fn test_jp_1nnn() {
        let mut cpu = Cpu::new(vec![0x11, 0x23], 0x0200);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x123);
    }

    #[test]
    fn test_call_2nnn() {
        let mut cpu = Cpu::new(vec![0x21, 0x23], 0x0200);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x123);
        assert_eq!(cpu.registers.sp, 1);
        assert_eq!(cpu.registers.stack[0], 0x0200);
    }
    #[test]
    fn test_call_2nnn_full() {
        let mut cpu = Cpu::new(vec![0x21, 0x23], 0x0200);
        cpu.registers.sp = 0xF;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x123);
        assert_eq!(cpu.registers.sp, 0);
        assert_eq!(cpu.registers.stack[0xF], 0x0200);
    }

    #[test]
    fn test_se_3xkk_no_skip() {
        let mut cpu = Cpu::new(vec![0x30, 0x55], 0x0200);
        cpu.registers.v[0x0] = 0x15;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
    }
    #[test]
    fn test_se_3xkk_skip() {
        let mut cpu = Cpu::new(vec![0x30, 0x55], 0x0200);
        cpu.registers.v[0x0] = 0x55;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0204);
    }

    #[test]
    fn test_sne_3xkk_no_skip() {
        let mut cpu = Cpu::new(vec![0x40, 0x55], 0x0200);
        cpu.registers.v[0x0] = 0x55;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
    }
    #[test]
    fn test_sne_4xkk_skip() {
        let mut cpu = Cpu::new(vec![0x40, 0x55], 0x0200);
        cpu.registers.v[0x0] = 0x15;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0204);
    }

    #[test]
    fn test_se_5xy0_no_skip() {
        let mut cpu = Cpu::new(vec![0x50, 0x10], 0x0200);
        cpu.registers.v[0x0] = 0x28;
        cpu.registers.v[0x1] = 0x55;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
    }
    #[test]
    fn test_se_5xy0_skip() {
        let mut cpu = Cpu::new(vec![0x50, 0x10], 0x0200);
        cpu.registers.v[0x0] = 0x15;
        cpu.registers.v[0x1] = 0x15;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0204);
    }

    #[test]
    fn test_ld_6xkk() {
        let mut cpu = Cpu::new(vec![0x60, 0x12], 0x0200);
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x12);
    }

    #[test]
    fn test_add_7xkk() {
        let mut cpu = Cpu::new(vec![0x70, 0x12], 0x0200);
        cpu.registers.v[0x0] = 0x33;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x45);
        assert_eq!(cpu.registers.v[0xF], 0); // Unchanged
    }

    #[test]
    fn test_ld_8xy0() {
        let mut cpu = Cpu::new(vec![0x80, 0x10], 0x0200);
        cpu.registers.v[0x0] = 0x12;
        cpu.registers.v[0x1] = 0x34;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x34);
        assert_eq!(cpu.registers.v[0x1], 0x34); // Unchanged...
    }

    #[test]
    fn test_or_8xy1() {
        let mut cpu = Cpu::new(vec![0x80, 0x11], 0x0200);
        cpu.registers.v[0x0] = 0b10101010;
        cpu.registers.v[0x1] = 0b01010101;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0xFF);
        assert_eq!(cpu.registers.v[0x1], 0b01010101);
    }

    #[test]
    fn test_and_8xy2() {
        let mut cpu = Cpu::new(vec![0x80, 0x12], 0x0200);
        cpu.registers.v[0x0] = 0b10101010;
        cpu.registers.v[0x1] = 0b01010101;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x0);
        assert_eq!(cpu.registers.v[0x1], 0b01010101);
    }

    #[test]
    fn test_xor_8xy3() {
        let mut cpu = Cpu::new(vec![0x80, 0x13], 0x0200);
        cpu.registers.v[0x0] = 0b10101111;
        cpu.registers.v[0x1] = 0b01011111;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0b11110000);
        assert_eq!(cpu.registers.v[0x1], 0b01011111);
    }

    #[test]
    fn test_add_8xy4_no_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x14], 0x0200);
        cpu.registers.v[0x0] = 0x22;
        cpu.registers.v[0x1] = 0x41;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x63);
        assert_eq!(cpu.registers.v[0x1], 0x41);
        assert_eq!(cpu.registers.v[0xF], 0x0);
    }
    #[test]
    fn test_add_8xy4_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x14], 0x0200);
        cpu.registers.v[0x0] = 0xF3;
        cpu.registers.v[0x1] = 0x41;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x34);
        assert_eq!(cpu.registers.v[0x1], 0x41);
        assert_eq!(cpu.registers.v[0xF], 0x1);
    }

    #[test]
    fn test_sub_8xy5_no_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x15], 0x0200);
        cpu.registers.v[0x0] = 0xF3;
        cpu.registers.v[0x1] = 0x20;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0xD3);
        assert_eq!(cpu.registers.v[0x1], 0x20);
        assert_eq!(cpu.registers.v[0xF], 0x0);
    }

    #[test]
    fn test_sub_8xy5_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x15], 0x0200);
        cpu.registers.v[0x0] = 0x25;
        cpu.registers.v[0x1] = 0x80;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0xA5); // Wraps
        assert_eq!(cpu.registers.v[0x1], 0x80);
        assert_eq!(cpu.registers.v[0xF], 0x1);
    }

    #[test]
    fn test_shr_8xy6_no_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x16], 0x0200);
        cpu.registers.v[0x0] = 0b01111110;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0b00111111);
        assert_eq!(cpu.registers.v[0xF], 0x0);
    }

    #[test]
    fn test_shr_8xy6_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x16], 0x0200);
        cpu.registers.v[0x0] = 0b00111111;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0b00011111);
        assert_eq!(cpu.registers.v[0xF], 0x1);
    }

    #[test]
    fn test_subn_8xy7_no_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x17], 0x0200);
        cpu.registers.v[0x0] = 0x25;
        cpu.registers.v[0x1] = 0x80;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x5B);
        assert_eq!(cpu.registers.v[0x1], 0x80);
        assert_eq!(cpu.registers.v[0xF], 0x0);
    }
    #[test]
    fn test_subn_8xy7_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x17], 0x0200);
        cpu.registers.v[0x0] = 0xF3;
        cpu.registers.v[0x1] = 0x20;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x2D); // Wraps
        assert_eq!(cpu.registers.v[0x1], 0x20);
        assert_eq!(cpu.registers.v[0xF], 0x1);
    }

    #[test]
    fn test_shl_8xye_no_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x1E], 0x0200);
        cpu.registers.v[0x0] = 0b01111110;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0b11111100);
        assert_eq!(cpu.registers.v[0xF], 0x0);
    }
    #[test]
    fn test_shl_8xye_carry() {
        let mut cpu = Cpu::new(vec![0x80, 0x1E], 0x0200);
        cpu.registers.v[0x0] = 0b11111100;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0b11111000);
        assert_eq!(cpu.registers.v[0xF], 0x1);
    }

    #[test]
    fn test_sne_9xy0_no_skip() {
        let mut cpu = Cpu::new(vec![0x90, 0x10], 0x0200);
        cpu.registers.v[0x0] = 0x12;
        cpu.registers.v[0x1] = 0x12;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
    }
    #[test]
    fn test_sne_9xy0_skip() {
        let mut cpu = Cpu::new(vec![0x90, 0x10], 0x0200);
        cpu.registers.v[0x0] = 0x12;
        cpu.registers.v[0x1] = 0x93;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0204);
    }

    #[test]
    fn test_ld_annn() {
        let mut cpu = Cpu::new(vec![0xa1, 0x23], 0x0200);
        cpu.tick();
        assert_eq!(cpu.registers.i, 0x123);
    }

    #[test]
    fn test_jp_bnnn() {
        let mut cpu = Cpu::new(vec![0xb4, 0x03], 0x0200);
        cpu.registers.v[0] = 0x53;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x456);
    }

    #[test]
    fn test_skp_ex9e_no_skip() {
        let mut cpu = Cpu::new(vec![0xE0, 0x9E], 0x0200);
        cpu.registers.v[0] = 0x6;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
    }
    #[test]
    fn test_skp_ex9e_skip() {
        let mut cpu = Cpu::new(vec![0xE0, 0x9E], 0x0200);
        cpu.registers.v[0] = 0x6;
        cpu.keypad.set_key(0x06, true);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0204);
    }

    #[test]
    fn test_skp_exa1_no_skip() {
        let mut cpu = Cpu::new(vec![0xE0, 0xA1], 0x0200);
        cpu.registers.v[0] = 0x6;
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0204);
    }
    #[test]
    fn test_skp_exa1_skip() {
        let mut cpu = Cpu::new(vec![0xE0, 0xA1], 0x0200);
        cpu.registers.v[0] = 0x6;
        cpu.keypad.set_key(0x06, true);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
    }

    #[test]
    fn test_ld_fx0a() {
        let mut cpu = Cpu::new(vec![0xF0, 0x0A], 0x0200);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0200);
        assert_eq!(cpu.registers.v[0], 0x0);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0200);
        assert_eq!(cpu.registers.v[0], 0x0);
        cpu.keypad.set_key(0x7, true);
        cpu.keypad.set_key(0x7, false);
        cpu.tick();
        assert_eq!(cpu.registers.pc, 0x0202);
        assert_eq!(cpu.registers.v[0], 0x7);
    }

    #[test]
    fn test_ld_fx1e() {
        let mut cpu = Cpu::new(vec![0xF0, 0x1E], 0x0200);
        cpu.registers.v[0] = 0x20;
        cpu.registers.i = 0x94;
        cpu.tick();
        assert_eq!(cpu.registers.v[0], 0x20);
        assert_eq!(cpu.registers.i, 0xB4);
    }

    #[test]
    fn test_ld_fx33() {
        let mut cpu = Cpu::new(vec![0xF0, 0x33], 0x0200);
        cpu.registers.v[0] = 0xC4; // 196
        cpu.registers.i = 0x500;
        cpu.tick();
        assert_eq!(cpu.memory.read(0x500), 1);
        assert_eq!(cpu.memory.read(0x501), 9);
        assert_eq!(cpu.memory.read(0x502), 6);
    }

    #[test]
    fn test_ld_fx55_first_four() {
        let mut cpu = Cpu::new(vec![0xF3, 0x55], 0x0200);
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;
        cpu.registers.v[2] = 0x56;
        cpu.registers.v[3] = 0x78;
        cpu.registers.v[4] = 0x9a;
        cpu.registers.i = 0x500;
        cpu.tick();
        assert_eq!(cpu.memory.read(0x500), 0x12);
        assert_eq!(cpu.memory.read(0x501), 0x34);
        assert_eq!(cpu.memory.read(0x502), 0x56);
        assert_eq!(cpu.memory.read(0x503), 0x78);
        assert_eq!(cpu.memory.read(0x504), 0x00);
    }
    #[test]
    fn test_ld_fx55_one() {
        let mut cpu = Cpu::new(vec![0xF0, 0x55], 0x0200);
        cpu.registers.v[0] = 0x12;
        cpu.registers.v[1] = 0x34;
        cpu.registers.i = 0x500;
        cpu.tick();
        assert_eq!(cpu.memory.read(0x500), 0x12);
        assert_eq!(cpu.memory.read(0x501), 0x00);
    }

    #[test]
    fn test_ld_fx65_first_four() {
        let mut cpu = Cpu::new(vec![0xF3, 0x65], 0x0200);
        cpu.memory.write(0x500, 0x12);
        cpu.memory.write(0x501, 0x34);
        cpu.memory.write(0x502, 0x56);
        cpu.memory.write(0x503, 0x78);
        cpu.memory.write(0x504, 0x9a);
        cpu.registers.i = 0x500;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x12);
        assert_eq!(cpu.registers.v[0x1], 0x34);
        assert_eq!(cpu.registers.v[0x2], 0x56);
        assert_eq!(cpu.registers.v[0x3], 0x78);
        assert_eq!(cpu.registers.v[0x4], 0x00);
    }
    #[test]
    fn test_ld_fx65_one() {
        let mut cpu = Cpu::new(vec![0xF0, 0x65], 0x0200);
        cpu.memory.write(0x500, 0x12);
        cpu.memory.write(0x501, 0x34);
        cpu.registers.i = 0x500;
        cpu.tick();
        assert_eq!(cpu.registers.v[0x0], 0x12);
        assert_eq!(cpu.registers.v[0x1], 0x0);
    }
}
