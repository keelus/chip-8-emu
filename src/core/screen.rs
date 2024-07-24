//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

// Must be a 64 x 32 screen, as for rows we are using u64(width) words.
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Screen(pub [u64; HEIGHT]);

impl Screen {
    pub fn new() -> Screen {
        Screen { 0: [0; HEIGHT] }
    }

    pub fn clear(&mut self) {
        self.0.fill(0);
    }
}
