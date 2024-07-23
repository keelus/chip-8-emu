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

pub struct Screen([u64; HEIGHT]);

impl Screen {
    pub fn new() -> Screen {
        Screen { 0: [0; HEIGHT] }
    }
}
