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
