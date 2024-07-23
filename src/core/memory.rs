//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

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
}
