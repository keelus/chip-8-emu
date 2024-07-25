//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use super::instruction::Instruction;

const MEMORY_SIZE: usize = 4096;

const HEX_SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

pub const HEX_SPRITES_WIDTH: u8 = 8;
pub const HEX_SPRITES_HEIGHT: u8 = 5;
pub const HEX_SPRITES_START_MEM: u16 = 0x0000;

// Memory structure:
// 0x200 - 0xFFF -> Program/ROM memory
// 0x000 - 0x1FF -> Interpreter specific
//
pub struct Memory([u8; MEMORY_SIZE]);

impl Memory {
    pub fn new(program: Vec<u8>, program_begin: u16) -> Memory {
        let mut mem = Memory {
            0: [0; MEMORY_SIZE],
        };

        let mut addr = HEX_SPRITES_START_MEM;
        for &hex_sprite in &HEX_SPRITES {
            for row in hex_sprite {
                mem.write(addr, row);
                addr += 1;
            }
        }

        for (index, &data) in program.iter().enumerate() {
            let (addr, overflows) = (program_begin).overflowing_add(index as u16);
            if overflows {
                panic!("Program read overflowed. Stopping.");
            }
            mem.write(addr, data);
        }

        mem
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.0[addr as usize] = data;
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let addr = addr as usize;

        let msb = self.0[addr] as u16;
        let lsb = self.0[addr + 1] as u16;

        msb << 8 | lsb
    }

    pub fn read_instruction(&self, addr: u16) -> Instruction {
        let data = self.read_u16(addr);

        let p1 = ((data >> 12) & 0xF) as u8;
        let p2 = ((data >> 8) & 0xF) as u8;
        let p3 = ((data >> 4) & 0xF) as u8;
        let p4 = (data & 0xF) as u8;

        Instruction::new((p1, p2, p3, p4))
    }
}

#[cfg(test)]
mod memory_tests {
    use super::Memory;

    #[test]
    fn test_read_instruction() {
        let mem = Memory::new(vec![0x12, 0x34], 0x0200);
        let instruction = mem.read_instruction(0x0200);
        assert_eq!(instruction.parts().0, 0x01);
        assert_eq!(instruction.parts().1, 0x02);
        assert_eq!(instruction.parts().2, 0x03);
        assert_eq!(instruction.parts().3, 0x04);
    }
}
