// It's bussin my g
use crate::{bios::BIOS, range::Range};

pub struct Bus {
    bios: BIOS,
}

impl Bus {
    pub fn new(bios: BIOS) -> Bus {
        Bus { bios }
    }

    pub fn load32(&self, addr: u32) -> u32 {
        expect_align(addr, 4);
        if let Some(offset) = BIOS::contains(addr) {
            return self.bios.load32(offset);
        }

        panic!("unhandled load32 at address 0x{:08X}", addr)
    }

    pub fn store32(&self, addr: u32, value: u32) {
        expect_align(addr, 4);
        println!("Storing 0x{:08X} to address 0x{:08X}", value, addr);

        if BIOS::contains(addr).is_some() {
            panic!("Illegal write to BIOS memory");
        } else if let Some(offset) = MEMCONTROL.contains(addr) {
            check_memcontrol(offset, value);
        } else if RAMSIZE.contains(addr).is_some() {
            println!("Ignoring write to RAMSIZE address range...");
        } else if CACHECONTROL.contains(addr).is_some() {
            println!("Ignoring write to CACHECONTROL address range...")
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

const MEMCONTROL: Range = Range(0x1f801000, 36);
const RAMSIZE: Range = Range(0x1f801060, 4);
const CACHECONTROL: Range = Range(0xfffe0130, 4);