// wagwan my g
use crate::bios::BIOS_START;
use crate::bus::Bus;
use crate::instruction::{ Instruction, RegisterIndex };
use log::{ debug, info };

pub struct CPU {
    pc: u32,
    next_pc: u32,
    // next_instruction: Instruction,
    counter: u32,
    pending_load: (RegisterIndex, u32),
    bus: Bus,

    sr: u32,
    hi: u32,
    lo: u32,

    input_registers: [u32; 32],
    output_registers: [u32; 32],
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        let mut registers = [0xcafebabe; 32];
        registers[0] = 0;

        CPU {
            pc: BIOS_START,
            next_pc: BIOS_START.wrapping_add(4),
            // next_instruction: Instruction{ value: 0x00 },
            bus,
            counter: 0,
            pending_load: (RegisterIndex(0), 0),

            sr: 0,
            hi: 0x42042069,
            lo: 0x42042069,
            input_registers: registers,
            output_registers: registers,
        }
    }

    fn load8(&self, addr: u32) -> u8 {
        self.bus.load8(addr)
    }

    fn load16(&self, addr: u32) -> u16 {
        self.bus.load16(addr)
    }

    fn load32(&self, addr: u32) -> u32 {
        self.bus.load32(addr)
    }

    fn store8(&mut self, addr: u32, value: u8) {
        self.bus.store8(addr, value);
    }

    fn store16(&mut self, addr: u32, value: u16) {
        self.bus.store16(addr, value);
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
        let mut pc = self.next_pc;
        pc = pc.wrapping_add(offset << 2);
        pc = pc.wrapping_sub(4);
        self.next_pc = pc;

    }

    fn decode_and_execute(&mut self, instruction: Instruction) {
        self.counter += 1;
        trace!("Executing instruction: 0x{:02X}", instruction.opcode());

        match instruction.opcode() {
            0x00 => {
                trace!("secondary: 0x{:02X}", instruction.secondary_opcode());
                match instruction.secondary_opcode() {
                    0x00 => self.op_sll(instruction),
                    0x02 => self.op_srl(instruction),
                    0x03 => self.op_sra(instruction),

                    0x08 => self.op_jr(instruction),
                    0x09 => self.op_jalr(instruction),
                    0x10 => self.op_mfhi(instruction),
                    0x11 => self.op_mthi(instruction),
                    0x12 => self.op_mflo(instruction),
                    0x13 => self.op_mtlo(instruction),
                    0x18 => self.op_mult(instruction),
                    0x19 => self.op_multu(instruction),
                    0x1A => self.op_div(instruction),
                    0x1B => self.op_divu(instruction),

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

            0x01 => self.op_bcondz(instruction),
            0x02 => self.op_j(instruction),
            0x03 => self.op_jal(instruction),
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

            0x20 => self.op_lb(instruction),
            0x21 => self.op_lh(instruction),
            0x23 => self.op_lw(instruction),
            0x24 => self.op_lbu(instruction),
            0x25 => self.op_lhu(instruction),

            0x28 => self.op_sb(instruction),
            0x29 => self.op_sh(instruction),
            0x2B => self.op_sw(instruction),

            _ => self.panic(instruction),
        }
    }

    fn op_bcondz(&mut self, instruction: Instruction) {
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs()) as i32;

        let ins_value = instruction.value;

        let is_bgez = (ins_value >> 16) & 1;
        let is_link = (ins_value >> 20) & 1 != 0;

        let test = ((source < 0) as u32) ^ is_bgez;

        if test != 0 {
            if is_link {
                self.set_register(RegisterIndex(31), self.next_pc);
            }

            self.branch(imm);
        }
    }

    fn op_j(&mut self, instruction: Instruction) {
        let imm = instruction.imm_jump() << 2;
        self.next_pc = (self.next_pc & 0xf0000000) | imm;
    }

    fn op_jal(&mut self, instruction: Instruction) {
        self.set_register(RegisterIndex(31), self.next_pc);
        self.op_j(instruction);
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
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        self.set_register(target, (left < right) as u32)
    }

    fn op_sltu(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        let result = left < right;
        self.set_register(target, result as u32)
    }

    fn op_cop0(&mut self, instruction: Instruction) {
        debug!("Executing cop0 instruction 0x{:08X}", instruction.cop_opcode());
        match instruction.cop_opcode() {
            0x00 => self.op_mfc0(instruction),
            0x04 => self.op_mtc0(instruction),
            _ => self.panic(instruction),
        }
    }

    fn op_mfc0(&mut self, instruction: Instruction) {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = match cop_r.0 {
            12 => self.sr,
            13 => panic!("Unhandled read from cop0 CAUSE register."),
            _ => panic!("Unhandled read from cop0 register."),
        };

        self.pending_load = (cpu_r, value);
    }

    fn op_mtc0(&mut self, instruction: Instruction) {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = self.register(cpu_r);

        match cop_r {
            RegisterIndex(3 | 5 | 6 | 7 | 9 | 11) => {
                if value != 0 {
                    self.panic_message(instruction, "Unhandled write to cop0 register.")
                }
            },
            RegisterIndex(12) => self.sr = value,
            RegisterIndex(13) => {
                if value != 0 {
                    self.panic_message(instruction, "Unhandled write to cop0 CAUSE register.")
                }
            }
            _ => self.panic_message(instruction, "Unhandled cop0 register.")
        }
    }

    fn op_sll(&mut self, instruction: Instruction) {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();
        self.set_register(destination, value << shift);
    }

    fn op_srl(&mut self, instruction: Instruction) {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();
        self.set_register(destination, value >> shift);
    }

    fn op_sra(&mut self, instruction: Instruction) {
        let destination = instruction.rd();
        let shift = instruction.imm5();
        let value = (self.register(instruction.rt()) as i32) >> shift;
        self.set_register(destination, value as u32);
    }

    fn op_jr(&mut self, instruction: Instruction) {
        let value = instruction.rs();
        self.next_pc = self.register(value);
    }

    fn op_jalr(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let value = self.register(instruction.rs());

        self.set_register(target, self.next_pc);

        self.next_pc = value;
    }

    fn op_mfhi(&mut self, instruction: Instruction) {
        let destination = instruction.rd();
        self.set_register(destination, self.hi);
    }

    fn op_mthi(&mut self, instruction: Instruction) {
        let source = self.register(instruction.rs());
        self.hi = source;
    }

    fn op_mflo(&mut self, instruction: Instruction) {
        let destination = instruction.rd();
        self.set_register(destination, self.lo);
    }

    fn op_mtlo(&mut self, instruction: Instruction) {
        let source = self.register(instruction.rs());
        self.lo = source;
    }

    fn op_mult(&mut self, instruction: Instruction) {
        panic!("mult")
    }

    fn op_multu(&mut self, instruction: Instruction) {
        panic!("multu")
    }

    fn op_div(&mut self, instruction: Instruction) {
        let dimmadome = self.register(instruction.rs()) as i32;
        let divisor = self.register(instruction.rt()) as i32;

        if divisor == 0 {
            self.hi = dimmadome as u32;

            if dimmadome >= 0 {
                self.lo = 0xffffffff;
            } else {
                self.lo = 1;
            }

        } else if dimmadome as u32 == 0x80000000 && divisor == -1 {
            self.hi = 0;
            self.lo = 0x80000000;
        } else {
            self.hi = (dimmadome % divisor) as u32;
            self.lo = (dimmadome / divisor) as u32;
        }
    }

    fn op_divu(&mut self, instruction: Instruction) {
        let dimmadome = self.register(instruction.rs());
        let divisor = self.register(instruction.rt());

        if dimmadome == 0 {
            self.hi = dimmadome;
            self.lo = 0xffffffff;
        } else {
            self.hi = dimmadome % divisor;
            self.lo = dimmadome / divisor;
        }
    }

    fn op_add(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        let value = match left.checked_add(right) {
            Some(value) => value as u32,
            None => panic!("ADD overflow")
        };

        self.set_register(target, value);
    }

    fn op_addu(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.set_register(target, left.wrapping_add(right));
    }

    fn op_sub(&mut self, instruction: Instruction) {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        let value = match left.checked_sub(right) {
            Some(value) => value as u32,
            None => panic!("ADD overflow")
        };

        self.set_register(target, value);
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
        let imm = instruction.imm16_se() as i32;
        let source = self.register(instruction.rs()) as i32;

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
        let imm = instruction.imm16_se() as i32;
        let source = self.register(instruction.rs()) as i32;
        let target = instruction.rt();

        self.set_register(target, (source < imm) as u32);
    }

    fn op_sltiu(&mut self, instruction: Instruction) {
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());
        let target = instruction.rt();

        self.set_register(target, (source < imm) as u32);
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

    fn op_sb(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return;
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.store8(base.wrapping_add(offset), value as u8);
    }

    fn op_sh(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return;
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.store16(base.wrapping_add(offset), value as u16);
    }

    fn op_sw(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return;
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.store32(base.wrapping_add(offset), value);
    }

    fn op_lb(&mut self, instruction: Instruction) {
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load8(base.wrapping_add(offset)) as i8;
        self.pending_load = (target_index, value as u32);
    }

    fn op_lh(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return;
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load16(base.wrapping_add(offset)) as i16;
        self.pending_load = (target_index, value as u32);
    }

    fn op_lw(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return;
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load32(base.wrapping_add(offset));
        self.pending_load = (target_index, value);
    }

    fn op_lbu(&mut self, instruction: Instruction) {
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load8(base.wrapping_add(offset));
        self.pending_load = (target_index, value as u32);

    }

    fn op_lhu(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return;
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load16(base.wrapping_add(offset));
        self.pending_load = (target_index, value as u32);

    }

    pub fn exec_next_instruction(&mut self) {
        let instruction = Instruction { value: self.load32(self.pc)};

        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        let (register, value) = self.pending_load;
        self.set_register(register, value);

        self.pending_load = (RegisterIndex(0), 0);

        self.decode_and_execute(instruction);
        self.input_registers = self.output_registers;
    }

    #[allow(dead_code)]
    fn dump_registers(&self) {
        debug!("Dumping registers:");
        for i in 0..self.input_registers.len() {
            debug!("    [{}] = 0x{:08X}", i, self.input_registers[i])
        }
    }

    fn panic(&self, instruction: Instruction) {
        // self.dump_registers();
        panic!(
            "Panicked at instruction: {} \nExecuted {} instructions.",
            instruction, self.counter
        );
    }

    fn panic_message(&self, instruction: Instruction, message: &str) {
        // self.dump_registers();
        panic!(
            "{} \nPanicked at instruction: {} \nExecuted {} instructions.",
            message, instruction, self.counter
        );
    }
}
