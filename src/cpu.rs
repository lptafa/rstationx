// wagwan my g
use crate::bios::BIOS_START;
use crate::bus::Bus;
use crate::instruction::{ Instruction, RegisterIndex };


pub struct CPU {
    pc: u32,
    next: Instruction,
    sr: u32,
    counter: u32,
    pending_load: (RegisterIndex, u32),
    bus: Bus,

    input_registers: [u32; 32],
    output_registers: [u32; 32],
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        let mut registers = [0xcafebabe; 32];
        registers[0] = 0;

        CPU {
            pc: BIOS_START,
            next: Instruction{ value: 0x00 },
            sr: 0,
            bus,
            counter: 0,
            input_registers: registers,
            output_registers: registers,
            pending_load: (RegisterIndex(0), 0),
        }
    }

    fn load32(&self, addr: u32) -> u32 {
        self.bus.load32(addr)
    }

    fn store32(&mut self, addr: u32, value: u32) {
        self.bus.store32(addr, value);
    }

    fn register(&self, index: RegisterIndex) -> u32 {
        self.input_registers[index.0 as usize]
    }

    fn set_register(&mut self, index: RegisterIndex, value: u32) {
        self.output_registers[index.0 as usize] = value;
        self.output_registers[0] = 0;
    }

    fn branch(&mut self, offset: u32) {
        let mut pc = self.pc;
        pc = pc.wrapping_add(offset << 2);
        pc = pc.wrapping_sub(4);
        self.pc = pc;

    }

    fn decode_and_execute(&mut self, instruction: Instruction) {
        self.counter += 1;
        println!("Executing: 0x{:02X}", instruction.opcode());

        match instruction.opcode() {
            0x00 => {
                println!("Executing secondary: 0x{:02X}", instruction.secondary_opcode());
                match instruction.secondary_opcode() {
                    0x00 => self.op_sll(instruction),

                    0x20 => self.op_add(instruction),
                    0x21 => self.op_addu(instruction),
                    0x22 => self.op_sub(instruction),
                    0x23 => self.op_subu(instruction),
                    0x24 => self.op_and(instruction),
                    0x25 => self.op_or(instruction),
                    0x26 => self.op_xor(instruction),
                    0x27 => self.op_nor(instruction),

                    0x2A => self.op_slt(instruction),
                    0x2B => self.op_sltu(instruction),

                    _ => self.panic(instruction),
                }
            }

            0x02 => self.op_j(instruction),

            0x04 => self.op_beq(instruction),
            0x05 => self.op_bne(instruction),
            0x06 => self.op_blez(instruction),
            0x07 => self.op_bgtz(instruction),
            0x08 => self.op_addi(instruction),
            0x09 => self.op_addiu(instruction),
            0x0A => self.op_slti(instruction),
            0x0B => self.op_sltiu(instruction),
            0x0C => self.op_andi(instruction),
            0x0D => self.op_ori(instruction),
            0x0E => self.op_xori(instruction),
            0x0F => self.op_lui(instruction),
            0x10 => self.op_cop0(instruction),
            0x11 => panic!("Call to missing coprocessor cop1"),
            0x12 => panic!("Unimplemented call to GTE"),
            0x13 => panic!("Call to missing coprocessor cop3"),

            0x23 => self.op_lw(instruction),

            0x2B => self.op_sw(instruction),
            _ => self.panic(instruction),
        }
    }

    fn op_j(&mut self, instruction: Instruction) {
        let imm = instruction.imm_jump() << 2;
        self.pc = (self.pc & 0xf0000000) | imm;
    }

    fn op_beq(&mut self, instruction: Instruction) {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        if left == right {
            self.branch(branch_offset);
        }
    }

    fn op_bne(&mut self, instruction: Instruction) {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        if left != right {
            self.branch(branch_offset);
        }
    }

    fn op_blez(&mut self, instruction: Instruction) {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());

        if left <= 0x0 {
            self.branch(branch_offset);
        }
    }

    fn op_bgtz(&mut self, instruction: Instruction) {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());

        if left > 0x0 {
            self.branch(branch_offset);
        }
    }

    fn op_slt(&mut self, instruction: Instruction) {
        // let target = instruction.rd();
        // let left = self.register(instruction.rs());
        // let right = self.register(instruction.rt());

        panic!("unhandled instruction slt");
    }

    fn op_sltu(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        let result = left < right;
        self.set_register(target, result as u32)
    }

    fn op_cop0(&mut self, instruction: Instruction) {
        println!("Executing cop0 instruction 0x{:08X}", instruction.cop_opcode());
        match instruction.cop_opcode() {
            0x04 => self.op_mtc0(instruction),
            _ => self.panic(instruction),
        }
    }

    fn op_mtc0(&mut self, instruction: Instruction) {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = self.register(cpu_r);

        match cop_r {
            RegisterIndex(3 | 5 | 6 | 7 | 9 | 11) => {
                if value != 0 {
                    self.panic_message(instruction, "Unhandled write to cop0r something xd")
                }
            },
            RegisterIndex(12) => self.sr = value,
            RegisterIndex(13) => {
                if value != 0 {
                    self.panic_message(instruction, "Unhandled write to CAUSE register")
                }
            }
            _ => self.panic_message(instruction, "Unhandled cop0 register")
        }
    }

    fn op_sll(&mut self, instruction: Instruction) {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();
        self.set_register(destination, value << shift);
    }

    fn op_add(&mut self, instruction: Instruction) {
        // let target = instruction.rd();
        // let left = self.register(instruction.rs());
        // let right = self.register(instruction.rt());

        // self.set_register(target, left & right);
        panic!();
    }

    fn op_addu(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, left.wrapping_add(right));
    }

    fn op_sub(&mut self, instruction: Instruction) {
        // let target = instruction.rd();
        // let left = self.register(instruction.rs());
        // let right = self.register(instruction.rt());

        // self.set_register(target, left & right);
        panic!();
    }

    fn op_subu(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, left.wrapping_sub(right));
    }

    fn op_and(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, left & right);
    }

    fn op_or(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, left | right);
    }

    fn op_xor(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, left ^ right);
    }

    fn op_nor(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, !(left | right));
    }

    fn op_addi(&mut self, instruction: Instruction) {
        let target = instruction.rt();
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());

        let value = match source.checked_add(imm) {
            Some(value) => value as u32,
            None => panic!("ADDI overflow")
        };

        self.set_register(target, value);

    }

    fn op_addiu(&mut self, instruction: Instruction) {
        let target = instruction.rt();
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());
        self.set_register(target, source.wrapping_add(imm));
    }

    fn op_slti(&mut self, instruction: Instruction) {
        println!("slti");
        self.panic(instruction);
    }

    fn op_sltiu(&mut self, instruction: Instruction) {
        println!("sltiu");
        self.panic(instruction);
    }

    fn op_andi(&mut self, instruction: Instruction) {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        self.set_register(target, self.register(source) & imm);
    }

    fn op_ori(&mut self, instruction: Instruction) {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        self.set_register(target, self.register(source) | imm);
    }

    fn op_xori(&mut self, instruction: Instruction) {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        self.set_register(target, self.register(source) ^ imm);
    }

    fn op_lui(&mut self, instruction: Instruction) {
        let target = instruction.rt();
        let value = instruction.imm16();

        self.set_register(target, value << 16);
    }

    fn op_sw(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            println!("ignoring store while cache is isolated");
            return;
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.store32(base.wrapping_add(offset), value);
    }

    fn op_lw(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            println!("ignoring load while cache is isolated");
            return;
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load32(base.wrapping_add(offset));
        self.pending_load = (target_index, value);
    }

    pub fn exec_next_instruction(&mut self) {
        let instruction = self.next;
        self.next = Instruction { value: self.load32(self.pc)};
        self.pc = self.pc.wrapping_add(4);

        let (register, value) = self.pending_load;
        self.set_register(register, value);

        self.pending_load = (RegisterIndex(0), 0);

        self.decode_and_execute(instruction);
        self.input_registers = self.output_registers;
    }

    #[allow(dead_code)]
    fn dump_registers(&self) {
        for i in 0..self.input_registers.len() {
            println!("Register {} = 0x{:08X}", i, self.input_registers[i])
        }
    }

    fn panic(&self, instruction: Instruction) {
        // self.dump_registers();
        panic!(
            "Panicked at instruction: {} \n Executed {} instructions.",
            instruction, self.counter
        );
    }

    fn panic_message(&self, instruction: Instruction, message: &str) {
        // self.dump_registers();
        panic!(
            "{} \nPanicked at instruction: {} \n Executed {} instructions.",
            message, instruction, self.counter
        );
    }
}
