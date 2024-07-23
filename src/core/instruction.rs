pub struct Instruction((u8, u8, u8, u8));

impl Instruction {
    pub fn new(code: (u8, u8, u8, u8)) -> Instruction {
        Instruction { 0: code }
    }

    pub fn parts(&self) -> (u8, u8, u8, u8) {
        self.0
    }

    pub fn x(&self) -> u8 {
        // 4 bit x-register
        self.0 .1
    }

    pub fn y(&self) -> u8 {
        // 4 bit y-register
        self.0 .2
    }

    pub fn n(&self) -> u8 {
        // 4 bit nibble
        self.0 .3
    }

    pub fn kk(&self) -> u8 {
        ((self.0 .2 as u8) << 4) | (self.0 .3 as u8)
    }

    pub fn nnn(&self) -> u16 {
        // 12 bits
        ((self.0 .1 as u16) << 8) | ((self.0 .2 as u16) << 4) | (self.0 .3 as u16)
    }
}
