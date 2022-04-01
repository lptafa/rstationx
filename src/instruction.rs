use core::fmt;

#[derive(Clone, Copy)]
pub struct RegisterIndex(pub u32);

#[derive(Clone, Copy)]
pub struct Instruction {
    pub value: u32,
}

impl Instruction {
    pub fn opcode(&self) -> u32 {
        self.value >> 26
    }

    pub fn secondary_opcode(&self) -> u32 {
        self.value & 0b111111
    }

    pub fn cop_opcode(&self) -> u32 {
        (self.value >> 21) & 0x1f
    }

    pub fn rs(&self) -> RegisterIndex {
        RegisterIndex((self.value >> 21) & 0x1f)
    }

    pub fn rt(&self) -> RegisterIndex {
        RegisterIndex((self.value >> 16) & 0x1f)
    }

    pub fn rd(&self) -> RegisterIndex {
        RegisterIndex((self.value >> 11) & 0x1f)
    }

    pub fn imm5(&self) -> u32 {
        (self.value >> 6) & 0x1f
    }

    pub fn imm16(&self) -> u32 {
        self.value & 0xffff
    }

    pub fn imm16_se(&self) -> u32 {
        ((self.value & 0xffff) as i16) as u32
    }

    pub fn imm_jump(&self) -> u32 {
        self.value & 0x3ffffff
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:08X} - {:032b}", self.value, self.value)
    }
}
