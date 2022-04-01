// It's bussin my g

const REGION_MASK: [u32; 8] = [
    // KUSEG: 2048MB
    0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
    // KSEG0:  512MB
    0x7fffffff,
    // KSEG1:  512MB
    0x1fffffff,
    // KSEG2: 1024MB
    0xffffffff, 0xffffffff,
];

const MEMCONTROL: Range = Range(0x1f801000, 36);
const RAMSIZE: Range = Range(0x1f801060, 4);
const CACHECONTROL: Range = Range(0xfffe0130, 4);
const SPU: Range = Range(0x1f801c00, 640);
const EXPANSION_1: Range = Range(0x1f000000, 8 * 1024 * 1024);
const EXPANSION_2: Range = Range(0x1f802000, 66);
const IRQ_CONTROL: Range = Range(0x1f801070, 8);

use crate::{bios::BIOS, range::Range, ram::RAM};

pub struct Bus {
    bios: BIOS,
    ram: RAM,
}

impl Bus {
    pub fn new(bios: BIOS, ram: RAM) -> Bus {
        Bus { bios, ram }

    }

    pub fn load8(&self, addr: u32) -> u8 {
        let addr = mask_region(addr);

        if let Some(offset) = BIOS::contains(addr) {
            return self.bios.load8(offset);
        } else if let Some(offset) = EXPANSION_1.contains(addr) {
            println!("unhandled load 8 at EXPANSION 1 range");
            return 0xff;
        } else if let Some(offset) = RAM::contains(addr) {
            return self.ram.load8(offset);
        }

        panic!("unhandled load8 at address 0x{:08X}", addr)
    }

    pub fn load16(&self, addr: u32) -> u16 {
        expect_align(addr, 2);

        let addr = mask_region(addr);

        panic!("unhandled load16 at address 0x{:08X}", addr)
    }

    pub fn load32(&self, addr: u32) -> u32 {
        expect_align(addr, 4);

        let addr = mask_region(addr);

        if let Some(offset) = BIOS::contains(addr) {
            return self.bios.load32(offset);
        } else if let Some(offset) = RAM::contains(addr) {
            return self.ram.load32(offset);
        }
        panic!("unhandled load32 at address 0x{:08X}", addr)
    }

    pub fn store8(&mut self, addr: u32, value: u8) {
        let addr = mask_region(addr);

        if let Some(offset) = EXPANSION_2.contains(addr) {
            println!("Unhandled write to expansion 2 register 0x{:08X}", offset);
            return;
        } else if let Some(offset) = RAM::contains(addr) {
            return self.ram.store8(offset, value)
        }
        panic!("Unhandled store8 0x{:08X} to address 0x{:08X}", value, addr);

    }

    pub fn store16(&mut self, addr: u32, value: u16) {
        expect_align(addr, 2);
        let addr = mask_region(addr);

        if let Some(offset) = SPU.contains(addr) {
            println!("Unhandled write to SPU register 0x{:08X}", offset);
            return;
        }
        panic!("Unhandled store16 0x{:08X} to address 0x{:08X}", value, addr);

    }

    pub fn store32(&mut self, addr: u32, value: u32) {
        expect_align(addr, 4);
        let addr = mask_region(addr);
        println!("Storing 0x{:08X} to address 0x{:08X}", value, addr);

        if BIOS::contains(addr).is_some() {
            panic!("Illegal write to BIOS memory");
        } else if let Some(offset) = MEMCONTROL.contains(addr) {
            check_memcontrol(offset, value);
        } else if RAMSIZE.contains(addr).is_some() {
            println!("Ignoring write to RAMSIZE address range...");
        } else if CACHECONTROL.contains(addr).is_some() {
            println!("Ignoring write to CACHECONTROL address range...")
        } else if let Some(offset) = RAM::contains(addr) {
            self.ram.store32(offset, value);
        } else if let Some(offset) = IRQ_CONTROL.contains(addr) {
            println!("Ignoring write to IRQ CONTROL range: 0x{:08X}", offset);
            return;
        } else {
            panic!("Unhandled write to address");
        }
    }
}

fn expect_align(addr: u32, align: u32) {
    if addr % align != 0 {
        panic!(
            "Unaligned memory access for address 0x{:08X}... exepected alignment of {}",
            addr, align
        );
    }
}

fn check_memcontrol(offset: u32, value: u32) {
    match offset {
        0 => {
            if value != 0x1f000000 {
                panic!("Bad expansion 1 base address: 0x{:08X}", value);
            }
        }
        4 => {
            if value != 0x1f802000 {
                panic!("Bad expansion 1 base address: 0x{:08X}", value);
            }
        }
        _ => println!("Unhandled write to MEMCONTROL register"),
    }
}

fn mask_region(addr: u32) -> u32 {
    let index = (addr >> 29) as usize;
    addr & REGION_MASK[index]
}
