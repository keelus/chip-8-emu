//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

mod core;
use core::cpu::Cpu;

const PROGRAM_BEGIN: u16 = 0x0200;

fn main() {
    let mut cpu = Cpu::new(vec![0x80, 0x32], PROGRAM_BEGIN);
    cpu.tick();
}
