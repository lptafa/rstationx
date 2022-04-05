// wagwan my g
use crate::bus::Bus;
use crate::instruction::{Instruction, RegisterIndex};
use crate::map::BIOS_START;
use log::debug;
use std::string::String;

pub struct CPU {
    pc: u32,
    current_pc: u32,
    next_pc: u32,
    counter: u32,
    pending_load: (RegisterIndex, u32),
    bus: Bus,

    sr: u32,
    hi: u32,
    lo: u32,
    cause: u32,
    epc: u32,

    branch: bool,
    delay: bool,

    input_registers: [u32; 32],
    output_registers: [u32; 32],
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        let mut registers = [0xcafebabe; 32];
        registers[0] = 0;

        CPU {
            pc: BIOS_START,
            current_pc: BIOS_START,
            next_pc: BIOS_START.wrapping_add(4),
            bus,
            counter: 0,
            pending_load: (RegisterIndex(0), 0),

            sr: 0,
            hi: 0x42042069,
            lo: 0x42042069,
            cause: 0x69696969,
            epc: 0xB00B1E5,

            branch: false,
            delay: false,

            input_registers: registers,
            output_registers: registers,
        }
    }

    fn load<T: TryFrom<u32>>(&self, addr: u32) -> Result<T, String> {
        self.bus.load(addr)
    }

    fn store<T: Into<u32>>(&mut self, addr: u32, value: T) -> Result<(), String> {
        self.bus.store(addr, value)
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

    fn decode_and_execute(&mut self, instruction: Instruction) -> Result<(), String> {
        self.counter += 1;
        trace!(
            "({}): Executing instruction: 0x{:02X}",
            self.counter,
            instruction.opcode()
        );

        return match instruction.opcode() {
            0x00 => {
                trace!("secondary: 0x{:02X}", instruction.secondary_opcode());
                match instruction.secondary_opcode() {
                    0x00 => self.op_sll(instruction),
                    0x02 => self.op_srl(instruction),
                    0x03 => self.op_sra(instruction),
                    0x04 => self.op_sllv(instruction),
                    // 0x07 => self.op_srav(instruction),
                    0x08 => self.op_jr(instruction),
                    0x09 => self.op_jalr(instruction),

                    0x0C => self.op_syscall(instruction),
                    0x0D => self.op_break(instruction),

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

                    op => Err(format!("Unhandled secondary opcode: 0x{:02x}", op)),
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
            0x11 => Err(String::from("Call to missing coprocessor cop1")),
            0x12 => Err(String::from("Unimplemented call to GTE")),
            0x13 => Err(String::from("Call to missing coprocessor cop3")),

            0x20 => self.op_lb(instruction),
            0x21 => self.op_lh(instruction),
            0x23 => self.op_lw(instruction),
            0x24 => self.op_lbu(instruction),
            0x25 => self.op_lhu(instruction),

            0x28 => self.op_sb(instruction),
            0x29 => self.op_sh(instruction),
            0x2B => self.op_sw(instruction),

            op => Err(format!("Unhandled opcode 0x{:02x}", op)),
        };
    }

    fn op_bcondz(&mut self, instruction: Instruction) -> Result<(), String> {
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
        Ok(())
    }

    fn op_j(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm_jump() << 2;
        self.next_pc = (self.next_pc & 0xf0000000) | imm;
        Ok(())
    }

    fn op_jal(&mut self, instruction: Instruction) -> Result<(), String> {
        self.set_register(RegisterIndex(31), self.next_pc);
        self.op_j(instruction)?;
        Ok(())
    }

    fn op_beq(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        if left == right {
            self.branch(branch_offset);
        }
        Ok(())
    }

    fn op_bne(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        if left != right {
            self.branch(branch_offset);
        }
        Ok(())
    }

    fn op_blez(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());

        if left <= 0x0 {
            self.branch(branch_offset);
        }
        Ok(())
    }

    fn op_bgtz(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());

        if left > 0x0 {
            self.branch(branch_offset);
        }
        Ok(())
    }

    fn op_slt(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        Ok(self.set_register(target, (left < right) as u32))
    }

    fn op_sltu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        let result = left < right;
        Ok(self.set_register(target, result as u32))
    }

    fn op_cop0(&mut self, instruction: Instruction) -> Result<(), String> {
        debug!(
            "Executing cop0 instruction 0x{:08X}",
            instruction.cop_opcode()
        );
        return match instruction.cop_opcode() {
            0x00 => self.op_mfc0(instruction),
            0x04 => self.op_mtc0(instruction),
            0x10 => self.op_rfe(instruction),
            _ => Err(String::from("Unhandled cop0 opcode")),
        };
    }

    fn op_mfc0(&mut self, instruction: Instruction) -> Result<(), String> {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = match cop_r.0 {
            12 => self.sr,
            13 => self.cause,
            14 => self.epc,
            _ => return Err(String::from("Unhandled read from cop0 register.")),
        };

        self.pending_load = (cpu_r, value);
        Ok(())
    }

    fn op_mtc0(&mut self, instruction: Instruction) -> Result<(), String> {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = self.register(cpu_r);

        match cop_r {
            RegisterIndex(3 | 5 | 6 | 7 | 9 | 11) => {
                if value != 0 {
                    return Err(String::from("Unhandled write to cop0 register."));
                }
            }
            RegisterIndex(12) => self.sr = value,
            RegisterIndex(13) => self.cause = value,
            RegisterIndex(14) => self.epc = value,
            _ => return Err(String::from("Unhandled cop0 register.")),
        }
        Ok(())
    }

    fn op_rfe(&mut self, instruction: Instruction) -> Result<(), String> {
        if instruction.value & 0x3f != 0x10 {
            return Err(String::from("Invalid cop0 instruction."));
        }

        // <magic version=2>
        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= mode >> 2;
        // </magic>
        Ok(())
    }

    fn op_sll(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();

        Ok(self.set_register(destination, value << shift))
    }

    fn op_srl(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();

        Ok(self.set_register(destination, value >> shift))
    }

    fn op_sra(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let shift = instruction.imm5();
        let value = (self.register(instruction.rt()) as i32) >> shift;

        Ok(self.set_register(destination, value as u32))
    }

    fn op_sllv(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let shift = self.register(instruction.rs());
        let value = self.register(instruction.rt()) << shift & 0x1f;

        Ok(self.set_register(destination, value))
    }

    // fn op_srav(&mut self, instruction: Instruction) -> Result<(), String> {
    //     let destination = instruction.rd();
    //     let shift = self.register(instruction.rs());
    //     let value = (self.register(instruction.rt()) as i32) >> shift & 0x1f;

    //     Ok(self.set_register(destination, value as u32))
    // }

    fn op_jr(&mut self, instruction: Instruction) -> Result<(), String> {
        let value = instruction.rs();
        self.next_pc = self.register(value);
        Ok(())
    }

    fn op_jalr(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let value = self.register(instruction.rs());

        self.set_register(target, self.next_pc);
        self.next_pc = value;
        Ok(())
    }

    fn op_syscall(&mut self, _instruction: Instruction) -> Result<(), String> {
        self.exception(Exception::Syscall)
    }

    fn op_break(&mut self, _instruction: Instruction) -> Result<(), String> {
        self.exception(Exception::Break)?;
        Err(String::from(
            "We don't know the exception number for break.",
        ))
    }

    fn op_mfhi(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        Ok(self.set_register(destination, self.hi))
    }

    fn op_mthi(&mut self, instruction: Instruction) -> Result<(), String> {
        let source = self.register(instruction.rs());
        self.hi = source;
        Ok(())
    }

    fn op_mflo(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        Ok(self.set_register(destination, self.lo))
    }

    fn op_mtlo(&mut self, instruction: Instruction) -> Result<(), String> {
        let source = self.register(instruction.rs());
        self.lo = source;
        Ok(())
    }

    fn op_mult(&mut self, _instruction: Instruction) -> Result<(), String> {
        Err(String::from("Unimplemented mult"))
    }

    fn op_multu(&mut self, _instruction: Instruction) -> Result<(), String> {
        Err(String::from("Unimplemented multu"))
    }

    fn op_div(&mut self, instruction: Instruction) -> Result<(), String> {
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
        Ok(())
    }

    fn op_divu(&mut self, instruction: Instruction) -> Result<(), String> {
        let dimmadome = self.register(instruction.rs());
        let divisor = self.register(instruction.rt());

        if dimmadome == 0 {
            self.hi = dimmadome;
            self.lo = 0xffffffff;
        } else {
            self.hi = dimmadome % divisor;
            self.lo = dimmadome / divisor;
        }
        Ok(())
    }

    fn op_add(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        let value = match left.checked_add(right) {
            Some(value) => value as u32,
            None => return Err(String::from("ADD overflow")),
        };

        Ok(self.set_register(target, value))
    }

    fn op_addu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        Ok(self.set_register(target, left.wrapping_add(right)))
    }

    fn op_sub(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        let value = match left.checked_sub(right) {
            Some(value) => value as u32,
            None => return Err(String::from("ADD overflow")),
        };

        Ok(self.set_register(target, value))
    }

    fn op_subu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        Ok(self.set_register(target, left.wrapping_sub(right)))
    }

    fn op_and(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        Ok(self.set_register(target, left & right))
    }

    fn op_or(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        Ok(self.set_register(target, left | right))
    }

    fn op_xor(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        Ok(self.set_register(target, left ^ right))
    }

    fn op_nor(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        Ok(self.set_register(target, !(left | right)))
    }

    fn op_addi(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16_se() as i32;
        let source = self.register(instruction.rs()) as i32;

        let value = match source.checked_add(imm) {
            Some(value) => value as u32,
            None => return Err(String::from("ADDI overflow")),
        };

        Ok(self.set_register(target, value))
    }

    fn op_addiu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());

        Ok(self.set_register(target, source.wrapping_add(imm)))
    }

    fn op_slti(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm16_se() as i32;
        let source = self.register(instruction.rs()) as i32;
        let target = instruction.rt();

        Ok(self.set_register(target, (source < imm) as u32))
    }

    fn op_sltiu(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());
        let target = instruction.rt();

        Ok(self.set_register(target, (source < imm) as u32))
    }

    fn op_andi(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        Ok(self.set_register(target, self.register(source) & imm))
    }

    fn op_ori(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        Ok(self.set_register(target, self.register(source) | imm))
    }

    fn op_xori(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        Ok(self.set_register(target, self.register(source) ^ imm))
    }

    fn op_lui(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let value = instruction.imm16();

        Ok(self.set_register(target, value << 16))
    }

    fn op_sb(&mut self, instruction: Instruction) -> Result<(), String> {
        if self.sr & 0x10000 != 0 {
            debug!("Ignoring load call while cache is isolated.");
            return Ok(());
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.store::<u8>(base.wrapping_add(offset), value as u8)
    }

    fn op_sh(&mut self, instruction: Instruction) -> Result<(), String> {
        if self.sr & 0x10000 != 0 {
            debug!("Ignoring load call while cache is isolated.");
            return Ok(());
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        if offset % 2 == 0 {
            self.store::<u16>(base.wrapping_add(offset), value as u16)
        } else {
            self.exception(Exception::AddressErrorStore)
        }
    }

    fn op_sw(&mut self, instruction: Instruction) -> Result<(), String> {
        if self.sr & 0x10000 != 0 {
            debug!("Ignoring load call while cache is isolated.");
            return Ok(());
        }

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        if offset % 4 == 0 {
            self.store::<u32>(base.wrapping_add(offset), value)
        } else {
            self.exception(Exception::AddressErrorStore)
        }
    }

    fn op_lb(&mut self, instruction: Instruction) -> Result<(), String> {
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load::<u8>(base.wrapping_add(offset))? as i8;
        self.pending_load = (target_index, value as u32);
        Ok(())
    }

    fn op_lh(&mut self, instruction: Instruction) -> Result<(), String> {
        if self.sr & 0x10000 != 0 {
            debug!("Ignoring load call while cache is isolated.");
            return Ok(());
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        if offset % 2 == 0 {
            let value = self.load::<u16>(base.wrapping_add(offset))? as i16;
            self.pending_load = (target_index, value as u32);
            Ok(())
        } else {
            self.exception(Exception::AddressErrorLoad)
        }
    }

    fn op_lw(&mut self, instruction: Instruction) -> Result<(), String> {
        if self.sr & 0x10000 != 0 {
            debug!("Ignoring load call while cache is isolated.");
            return Ok(());
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        if offset % 4 == 0 {
            let value = self.load::<u32>(base.wrapping_add(offset))?;
            self.pending_load = (target_index, value);
            Ok(())
        } else {
            self.exception(Exception::AddressErrorLoad)
        }
    }

    fn op_lbu(&mut self, instruction: Instruction) -> Result<(), String> {
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load::<u8>(base.wrapping_add(offset))?;
        self.pending_load = (target_index, value as u32);
        Ok(())
    }

    fn op_lhu(&mut self, instruction: Instruction) -> Result<(), String> {
        if self.sr & 0x10000 != 0 {
            debug!("Ignoring load call while cache is isolated.");
            return Ok(());
        }

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load::<u16>(base.wrapping_add(offset))?;
        self.pending_load = (target_index, value as u32);
        Ok(())
    }

    fn exception(&mut self, cause: Exception) -> Result<(), String> {
        let handler = match self.sr & (1 << 22) != 0 {
            true => 0xbfc00180,
            false => 0x80000080,
        };

        // <magic version=1>
        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= (mode << 2) & 0x3f;
        // </magic>

        self.cause = (cause as u32) << 2;
        self.epc = self.current_pc;

        self.pc = handler;
        self.next_pc = self.pc.wrapping_add(4);

        Ok(())
    }

    pub fn exec_next_instruction(&mut self) {
        self.current_pc = self.pc;

        if self.current_pc % 4 != 0 {
            self.exception(Exception::AddressErrorLoad).unwrap();
            return;
        }

        let instruction = Instruction {
            value: self
                .load::<u32>(self.pc)
                .expect("Failed to load instruction from self.pc"),
        };

        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        self.delay = self.branch;
        self.branch = false;

        // <unsure>
        let (register, value) = self.pending_load;
        self.set_register(register, value);

        self.pending_load = (RegisterIndex(0), 0);
        // </unsure>

        if let Err(msg) = self.decode_and_execute(instruction) {
            self.panic_message(instruction, msg.as_str());
        }
        self.input_registers = self.output_registers;
    }

    #[allow(dead_code)]
    fn dump_registers(&self) {
        debug!("Dumping registers:");
        for i in 0..self.input_registers.len() {
            debug!("    [{}] = 0x{:08X}", i, self.input_registers[i])
        }
    }

    fn panic_message(&self, instruction: Instruction, message: &str) {
        self.dump_registers();
        eprintln!("----------------------------------------------------------------");
        eprintln!("[-] Instruction: {}", instruction.value);
        eprintln!("[-] NOTE: {}", message);
        eprintln!("[-] Executed {} instructions", self.counter);
        eprintln!("----------------------------------------------------------------");
        panic!();
    }
}

#[allow(dead_code)]
enum Exception {
    Interrupt = 0x0,
    AddressErrorLoad = 0x4,
    AddressErrorStore = 0x5,
    BusErrorFetch = 0x6,
    BusErrorLoad = 0x7,
    Syscall = 0x8,
    Break = 0x9,
    Reserved = 0xA,
    CopUnusable = 0xB,
    AOverflow = 0xC,
}
