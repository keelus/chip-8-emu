//  _             _
// | |           | |
// | | _____  ___| |_   _ ___
// | |/ / _ \/ _ \ | | | / __|
// |   <  __/  __/ | |_| \__ \
// |_|\_\___|\___|_|\__,_|___/
//
// https://github.com/keelus/chip-8-emu

use std::collections::HashMap;

// Keypad:
// 1 2 3 C    1 2 3 4
// 4 5 6 D -> Q W E R
// 7 8 9 E -> A S D F
// A 0 B F    Z X C V
//
pub struct Keypad {
    key_map: HashMap<u8, bool>, // Down = true, Up = false
    pub last_key: Option<u8>,
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            key_map: HashMap::from((0x0..=0xF).map(|i| (i, false)).collect::<HashMap<_, _>>()),
            last_key: None,
        }
    }

    pub fn set_key(&mut self, idx: u8, state: bool) {
        let entry = self.key_map.get_mut(&idx).unwrap();

        if *entry && !state {
            // Key released
            self.last_key = Some(idx);
        }

        *entry = state;
    }

    pub fn get_key_state(&mut self, idx: u8) -> bool {
        let state = *self.key_map.get(&idx).unwrap();
        self.last_key = None;
        state
    }

    pub fn get_released_key(&mut self) -> Option<u8> {
        let last_key = self.last_key;
        self.last_key = None;
        last_key
    }
}
