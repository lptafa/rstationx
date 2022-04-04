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

    fn instruction(&self) -> &'static str {
        match self.opcode() {
            0x00 => match self.secondary_opcode() {
                0x00 => "sll",
                0x02 => "srl",
                0x03 => "sra",
                0x04 => "sllv",
                0x08 => "jr",
                0x09 => "jalr",
                0x0C => "syscall",
                0x0D => "break",
                0x10 => "mfhi",
                0x11 => "mthi",
                0x12 => "mflo",
                0x13 => "mtlo",
                0x18 => "mult",
                0x19 => "multu",
                0x1A => "div",
                0x1B => "divu",
                0x20 => "add",
                0x21 => "addu",
                0x22 => "sub",
                0x23 => "subu",
                0x24 => "and",
                0x25 => "or",
                0x26 => "xor",
                0x27 => "nor",
                0x2A => "slt",
                0x2B => "sltu",
                _ => "Invalid secondary opcode",
            },
            0x01 => "bcondz",
            0x02 => "j",
            0x03 => "jal",
            0x04 => "beq",
            0x05 => "bne",
            0x06 => "blez",
            0x07 => "bgtz",
            0x08 => "addi",
            0x09 => "addiu",
            0x0A => "slti",
            0x0B => "sltiu",
            0x0C => "andi",
            0x0D => "ori",
            0x0E => "xori",
            0x0F => "lui",
            0x10 => "cop0",
            0x20 => "lb",
            0x21 => "lh",
            0x23 => "lw",
            0x24 => "lbu",
            0x25 => "lhu",
            0x28 => "sb",
            0x29 => "sh",
            0x2B => "sw",
            _ => "Invalid opcode",
        }
    }
}
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:08X} - {}", self.value, self.instruction())
    }
}
