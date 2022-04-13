// wagwan my g
use crate::bus::Bus;
use crate::instruction::{Instruction, RegisterIndex};
use crate::map::BIOS_START;
use crate::utils::Error;
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

    registers: [u32; 32],
}

macro_rules! ignore_cache {
    ($self:ident) => {
        if $self.sr & 0x10000 != 0 {
            trace!("Ignoring load call while cache is isolated.");
            return Ok(());
        }
    };
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

            registers,
        }
    }

    fn load<T: TryFrom<u32>>(&self, addr: u32) -> Result<T, String> {
        self.bus.load(addr)
    }

    fn store<T: Into<u32>>(&mut self, addr: u32, value: T) -> Result<(), String> {
        self.bus.store(addr, value)
    }

    fn register(&self, index: RegisterIndex) -> u32 {
        self.registers[index.0 as usize]
    }

    fn set_register(&mut self, index: RegisterIndex, value: u32) {
        self.registers[index.0 as usize] = value;
        self.registers[0] = 0;
    }

    fn delayed_load(&mut self) {
        let (index, value) = self.pending_load;
        self.set_register(index, value);
        self.pending_load = (RegisterIndex(0), 0);
    }

    fn delayed_load_chain(&mut self, reg: RegisterIndex, val: u32) {
        let (oreg, oval) = self.pending_load;
        if reg.0 != oreg.0 {
            self.set_register(oreg, oval);
        }
        self.pending_load = (reg, val);
    }

    fn branch(&mut self, offset: u32) {
        let mut pc = self.next_pc;
        pc = pc.wrapping_add(offset << 2);
        pc = pc.wrapping_sub(4);
        self.next_pc = pc;
        self.branch = true;
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
                    0x06 => self.op_srlv(instruction),
                    0x07 => self.op_srav(instruction),
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

                    _ => self.exception(Exception::IllegalInstruction),
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

            // COP-0
            0x10 => match instruction.cop_opcode() {
                0x00 => self.op_mfc0(instruction),
                0x04 => self.op_mtc0(instruction),
                0x10 => self.op_rfe(instruction),
                _ => Error!(
                    "Unhandled cop0 instruction: 0x{:08X}",
                    instruction.cop_opcode()
                ),
            },

            0x11 => self.exception(Exception::CoprocessorError), // COP1
            0x12 => self.op_cop2(instruction),
            0x13 => self.exception(Exception::CoprocessorError), // COP3

            0x20 => self.op_lb(instruction),
            0x21 => self.op_lh(instruction),
            0x22 => self.op_lwl(instruction),
            0x23 => self.op_lw(instruction),
            0x24 => self.op_lbu(instruction),
            0x25 => self.op_lhu(instruction),
            0x26 => self.op_lwr(instruction),

            0x28 => self.op_sb(instruction),
            0x29 => self.op_sh(instruction),
            0x2A => self.op_swl(instruction),
            0x2B => self.op_sw(instruction),
            0x2E => self.op_swr(instruction),

            0x30 => self.exception(Exception::IllegalInstruction), // LWC0
            0x31 => self.exception(Exception::IllegalInstruction), // LWC1
            0x32 => self.op_lwc2(instruction),                     // LWC2
            0x33 => self.exception(Exception::IllegalInstruction), // LWC3

            0x38 => self.exception(Exception::IllegalInstruction), // SWC0
            0x39 => self.exception(Exception::IllegalInstruction), // SWC1
            0x3A => self.op_swc2(instruction),                     // SWC2
            0x3B => self.exception(Exception::IllegalInstruction), // SWC3

            _ => self.exception(Exception::IllegalInstruction),
        };
    }

    fn op_bcondz(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs()) as i32;

        let ins_value = instruction.value;

        let is_bgez = (ins_value >> 16) & 1;
        let is_link = ((ins_value >> 20) & 1) != 0;

        let test = (source < 0) as u32;
        let test = test ^ is_bgez;

        self.delayed_load();

        if is_link {
            self.set_register(RegisterIndex(31), self.next_pc);
        }
        if test != 0 {
            self.branch(imm);
        }
        Ok(())
    }

    fn op_j(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm_jump() << 2;
        self.next_pc = (self.next_pc & 0xf0000000) | imm;
        self.branch = true;
        self.delayed_load();
        Ok(())
    }

    fn op_jal(&mut self, instruction: Instruction) -> Result<(), String> {
        let ra = self.next_pc;
        self.op_j(instruction)?;
        self.set_register(RegisterIndex(31), ra);
        self.branch = true;
        Ok(())
    }

    fn op_beq(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        if left == right {
            self.branch(branch_offset);
        }

        self.delayed_load();
        Ok(())
    }

    fn op_bne(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        if left != right {
            self.branch(branch_offset);
        }
        self.delayed_load();
        Ok(())
    }

    fn op_blez(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs()) as i32;

        if left <= 0x0 {
            self.branch(branch_offset);
        }
        self.delayed_load();
        Ok(())
    }

    fn op_bgtz(&mut self, instruction: Instruction) -> Result<(), String> {
        let branch_offset = instruction.imm16_se();
        let left = self.register(instruction.rs()) as i32;

        if left > 0x0 {
            self.branch(branch_offset);
        }
        self.delayed_load();
        Ok(())
    }

    fn op_slt(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        self.delayed_load();

        Ok(self.set_register(target, (left < right) as u32))
    }

    fn op_sltu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        let result = left < right;
        Ok(self.set_register(target, result as u32))
    }

    fn op_cop2(&mut self, instruction: Instruction) -> Result<(), String> {
        Error!("Unimplemented cop2 opcode: {}", instruction)
    }

    fn op_mfc0(&mut self, instruction: Instruction) -> Result<(), String> {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = match cop_r.0 {
            12 => self.sr,
            13 => self.cause,
            14 => self.epc,
            _ => return Error!("Unhandled read from cop0 register."),
        };

        self.delayed_load_chain(cpu_r, value);
        Ok(())
    }

    fn op_mtc0(&mut self, instruction: Instruction) -> Result<(), String> {
        let cpu_r = instruction.rt();
        let cop_r = instruction.rd();

        let value = self.register(cpu_r);

        self.delayed_load();

        match cop_r {
            RegisterIndex(3 | 5 | 6 | 7 | 9 | 11) => {
                if value != 0 {
                    return Error!("Unhandled write to cop0 register.");
                }
            }
            RegisterIndex(12) => self.sr = value,
            RegisterIndex(13) => self.cause = value,
            RegisterIndex(14) => self.epc = value,
            _ => return Error!("Unhandled cop0 register."),
        }
        Ok(())
    }

    fn op_rfe(&mut self, instruction: Instruction) -> Result<(), String> {
        if instruction.value & 0x3f != 0x10 {
            return Error!("Invalid cop0 instruction.");
        }

        self.delayed_load();

        // <magic version=2>
        let mode = self.sr & 0x3f;
        self.sr &= !0xf;
        self.sr |= mode >> 2;
        // </magic>
        Ok(())
    }

    fn op_sll(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();

        self.delayed_load();

        Ok(self.set_register(destination, value << shift))
    }

    fn op_srl(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let value = self.register(instruction.rt());
        let shift = instruction.imm5();

        self.delayed_load();

        Ok(self.set_register(destination, value >> shift))
    }

    fn op_sra(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let shift = instruction.imm5();
        let value = (self.register(instruction.rt()) as i32) >> shift;

        self.delayed_load();

        Ok(self.set_register(destination, value as u32))
    }

    fn op_sllv(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let shift = self.register(instruction.rs());
        let value = self.register(instruction.rt()) << shift & 0x1f;

        self.delayed_load();

        Ok(self.set_register(destination, value))
    }

    fn op_srlv(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let shift = self.register(instruction.rs());
        let value = (self.register(instruction.rt()) as u32) >> shift & 0x1f;

        self.delayed_load();

        Ok(self.set_register(destination, value as u32))
    }

    fn op_srav(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let shift = self.register(instruction.rs());
        let value = (self.register(instruction.rt()) as i32) >> shift & 0x1f;

        self.delayed_load();

        Ok(self.set_register(destination, value as u32))
    }

    fn op_jr(&mut self, instruction: Instruction) -> Result<(), String> {
        let value = instruction.rs();
        self.next_pc = self.register(value);
        self.branch = true;
        self.delayed_load();
        Ok(())
    }

    fn op_jalr(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let value = self.register(instruction.rs());
        let ra = self.next_pc;
        self.next_pc = value;

        self.delayed_load();

        self.set_register(target, ra);
        self.branch = true;

        Ok(())
    }

    fn op_syscall(&mut self, _instruction: Instruction) -> Result<(), String> {
        self.exception(Exception::Syscall)
    }

    fn op_break(&mut self, _instruction: Instruction) -> Result<(), String> {
        self.exception(Exception::Break)
    }

    fn op_mfhi(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let hi = self.hi;

        self.delayed_load();

        Ok(self.set_register(destination, hi))
    }

    fn op_mthi(&mut self, instruction: Instruction) -> Result<(), String> {
        let source = self.register(instruction.rs());
        self.hi = source;
        self.delayed_load();
        Ok(())
    }

    fn op_mflo(&mut self, instruction: Instruction) -> Result<(), String> {
        let destination = instruction.rd();
        let lo = self.lo;

        self.delayed_load();

        Ok(self.set_register(destination, lo))
    }

    fn op_mtlo(&mut self, instruction: Instruction) -> Result<(), String> {
        let source = self.register(instruction.rs());
        self.lo = source;
        self.delayed_load();
        Ok(())
    }

    fn op_mult(&mut self, instruction: Instruction) -> Result<(), String> {
        let a = self.register(instruction.rs()) as i64;
        let b = self.register(instruction.rt()) as i64;

        self.delayed_load();

        self.hi = ((a * b) >> 32) as u32;
        self.lo = (a * b) as u32;
        Ok(())
    }

    fn op_multu(&mut self, instruction: Instruction) -> Result<(), String> {
        let a = self.register(instruction.rs()) as u64;
        let b = self.register(instruction.rt()) as u64;

        self.delayed_load();

        self.hi = ((a * b) >> 32) as u32;
        self.lo = (a * b) as u32;
        Ok(())
    }

    fn op_div(&mut self, instruction: Instruction) -> Result<(), String> {
        let dimmadome = self.register(instruction.rs()) as i32;
        let divisor = self.register(instruction.rt()) as i32;

        self.delayed_load();

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

        self.delayed_load();

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

        self.delayed_load();

        return match left.checked_add(right) {
            Some(value) => Ok(self.set_register(target, value as u32)),
            None => self.exception(Exception::Overflow),
        };
    }

    fn op_addu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        Ok(self.set_register(target, left.wrapping_add(right)))
    }

    fn op_sub(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs()) as i32;
        let right = self.register(instruction.rt()) as i32;

        self.delayed_load();

        return match left.checked_sub(right) {
            Some(value) => Ok(self.set_register(target, value as u32)),
            None => self.exception(Exception::Overflow),
        };
    }

    fn op_subu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        Ok(self.set_register(target, left.wrapping_sub(right)))
    }

    fn op_and(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        Ok(self.set_register(target, left & right))
    }

    fn op_or(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        Ok(self.set_register(target, left | right))
    }

    fn op_xor(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        Ok(self.set_register(target, left ^ right))
    }

    fn op_nor(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rd();
        let left = self.register(instruction.rs());
        let right = self.register(instruction.rt());

        self.delayed_load();

        Ok(self.set_register(target, !(left | right)))
    }

    fn op_addi(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16_se() as i32;
        let source = self.register(instruction.rs()) as i32;

        self.delayed_load();

        return match source.checked_add(imm) {
            Some(value) => Ok(self.set_register(target, value as u32)),
            None => self.exception(Exception::Overflow),
        };
    }

    fn op_addiu(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());

        self.delayed_load();

        Ok(self.set_register(target, source.wrapping_add(imm)))
    }

    fn op_slti(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm16_se() as i32;
        let source = self.register(instruction.rs()) as i32;
        let target = instruction.rt();

        self.delayed_load();

        Ok(self.set_register(target, (source < imm) as u32))
    }

    fn op_sltiu(&mut self, instruction: Instruction) -> Result<(), String> {
        let imm = instruction.imm16_se();
        let source = self.register(instruction.rs());
        let target = instruction.rt();

        self.delayed_load();

        Ok(self.set_register(target, (source < imm) as u32))
    }

    fn op_andi(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        self.delayed_load();

        Ok(self.set_register(target, self.register(source) & imm))
    }

    fn op_ori(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        self.delayed_load();

        Ok(self.set_register(target, self.register(source) | imm))
    }

    fn op_xori(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let imm = instruction.imm16();
        let source = instruction.rs();

        self.delayed_load();

        Ok(self.set_register(target, self.register(source) ^ imm))
    }

    fn op_lui(&mut self, instruction: Instruction) -> Result<(), String> {
        let target = instruction.rt();
        let value = instruction.imm16();

        self.delayed_load();

        Ok(self.set_register(target, value << 16))
    }

    fn op_sb(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.delayed_load();

        self.store::<u8>(base.wrapping_add(offset), value as u8)
    }

    fn op_sh(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.delayed_load();

        if offset % 2 == 0 {
            self.store::<u16>(base.wrapping_add(offset), value as u16)
        } else {
            self.exception(Exception::AddressErrorStore)
        }
    }

    fn op_sw(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        self.delayed_load();

        if offset % 4 == 0 {
            self.store::<u32>(base.wrapping_add(offset), value)
        } else {
            self.exception(Exception::AddressErrorStore)
        }
    }

    fn op_swl(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        let addr = base.wrapping_add(offset);
        let aligned_addr = addr & !0b11;
        let aligned_word = self.load::<u32>(aligned_addr)?;

        let new_val = match addr & 0b11 {
            0 => (aligned_word & 0xffff_ff00) | (value >> 24),
            1 => (aligned_word & 0xffff_0000) | (value >> 16),
            2 => (aligned_word & 0xff00_0000) | (value >> 8),
            3 => (aligned_word & 0x0000_0000) | (value >> 0),
            _ => unreachable!(),
        };

        self.delayed_load();

        self.store::<u32>(aligned_addr, new_val)
    }

    fn op_swr(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let value = self.register(instruction.rt());
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());

        let addr = base.wrapping_add(offset);
        let aligned_addr = addr & !0b11;
        let aligned_word = self.load::<u32>(aligned_addr)?;

        let new_val = match addr & 0b11 {
            0 => (aligned_word & 0x0000_0000) | (value << 0),
            1 => (aligned_word & 0x0000_00ff) | (value << 8),
            2 => (aligned_word & 0x0000_ffff) | (value << 16),
            3 => (aligned_word & 0x00ff_ffff) | (value << 24),
            _ => unreachable!(),
        };

        self.delayed_load();

        self.store::<u32>(aligned_addr, new_val)
    }

    fn op_lb(&mut self, instruction: Instruction) -> Result<(), String> {
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = (self.load::<u8>(base.wrapping_add(offset))? as i8) as i32;
        self.delayed_load_chain(target_index, value as u32);
        Ok(())
    }

    fn op_lh(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        if offset % 2 == 0 {
            let value = (self.load::<u16>(base.wrapping_add(offset))? as i16) as i32;
            self.delayed_load_chain(target_index, value as u32);
            Ok(())
        } else {
            self.exception(Exception::AddressErrorLoad)
        }
    }

    fn op_lw(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        if offset % 4 == 0 {
            let value = self.load::<u32>(base.wrapping_add(offset))?;
            self.delayed_load_chain(target_index, value as u32);
            Ok(())
        } else {
            self.exception(Exception::AddressErrorLoad)
        }
    }

    fn op_lwl(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let addr = base.wrapping_add(offset);

        let target_index = instruction.rt();

        let (preg, pval) = self.pending_load;
        let cur_v = if preg.0 == target_index.0 {
            pval
        } else {
            self.register(target_index)
        };

        let aligned_addr = addr & !0x3;
        let aligned_word = self.load::<u32>(aligned_addr)?;

        let value = match addr & 0b11 {
            0 => (cur_v & 0x00FF_FFFF) | (aligned_word << 24),
            1 => (cur_v & 0x0000_FFFF) | (aligned_word << 16),
            2 => (cur_v & 0x0000_00FF) | (aligned_word << 8),
            3 => (cur_v & 0x0000_0000) | (aligned_word << 0),
            _ => unreachable!(),
        };
        self.delayed_load_chain(target_index, value);
        Ok(())
    }

    fn op_lwr(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let addr = base.wrapping_add(offset);

        let target_index = instruction.rt();

        let (preg, pval) = self.pending_load;
        let cur_v = if preg.0 == target_index.0 {
            pval
        } else {
            self.register(target_index)
        };

        let aligned_addr = addr & !0x3;
        let aligned_word = self.load::<u32>(aligned_addr)?;

        let value = match addr & 0b11 {
            0 => (cur_v & 0x0000_0000) | (aligned_word >> 0),
            1 => (cur_v & 0xFF00_0000) | (aligned_word >> 8),
            2 => (cur_v & 0xFFFF_0000) | (aligned_word >> 16),
            3 => (cur_v & 0xFFFF_FF00) | (aligned_word >> 24),
            _ => unreachable!(),
        };
        self.delayed_load_chain(target_index, value);
        Ok(())
    }

    fn op_lbu(&mut self, instruction: Instruction) -> Result<(), String> {
        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load::<u8>(base.wrapping_add(offset))?;
        self.delayed_load_chain(target_index, value as u32);
        Ok(())
    }

    fn op_lhu(&mut self, instruction: Instruction) -> Result<(), String> {
        ignore_cache!(self);

        let base = instruction.imm16_se();
        let offset = self.register(instruction.rs());
        let target_index = instruction.rt();

        let value = self.load::<u16>(base.wrapping_add(offset))?;
        self.delayed_load_chain(target_index, value as u32);
        Ok(())
    }

    fn op_lwc2(&mut self, instruction: Instruction) -> Result<(), String> {
        Error!("Unimplemented instruction: {}", instruction)
    }

    fn op_swc2(&mut self, instruction: Instruction) -> Result<(), String> {
        Error!("Unimplemented instruction: {}", instruction)
    }

    fn exception(&mut self, cause: Exception) -> Result<(), String> {
        self.delayed_load();

        let handler = match self.sr & (1 << 22) != 0 {
            true => 0x80000180,
            false => 0x80000080,
        };

        // <magic version=1>
        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= (mode << 2) & 0x3f;
        // </magic>

        self.cause &= !0x7c;
        self.cause = (cause as u32) << 2;

        if self.delay {
            self.epc = self.current_pc.wrapping_add(4);
            self.cause |= 1 << 31;
        } else {
            self.epc = self.current_pc;
            self.cause &= !(1 << 31);
        }

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

        if let Err(msg) = self.decode_and_execute(instruction) {
            self.panic_message(instruction, msg.as_str());
        }
    }

    #[allow(dead_code)]
    fn dump_registers(&self) {
        debug!("Dumping registers:");
        for i in 0..self.registers.len() {
            debug!("    [{}] = 0x{:08X}", i, self.registers[i])
        }
    }

    fn panic_message(&self, instruction: Instruction, message: &str) {
        self.dump_registers();
        eprintln!("----------------------------------------------------------------");
        eprintln!("[-] Instruction: {}", instruction);
        eprintln!("[-] Executed {} instructions", self.counter);
        eprintln!("{}", message);
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
    IllegalInstruction = 0xA,
    CoprocessorError = 0xB,
    Overflow = 0xC,
}
