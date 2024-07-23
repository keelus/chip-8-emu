//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display([u64; HEIGHT]);

impl Display {
    pub fn new() -> Display {
        Display { 0: [0; HEIGHT] }
    }
}
