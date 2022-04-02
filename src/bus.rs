// It's bussin my g
use log::debug;

use crate::bios::BIOS;
use crate::map;
use crate::map::MemoryRegion;
use crate::ram::RAM;
use crate::utils;

pub struct Bus {
    bios: BIOS,
    ram: RAM,
}

impl Bus {
    pub fn new(bios: BIOS, ram: RAM) -> Bus {
        Bus { bios, ram }
    }

    pub fn load8(&self, addr: u32) -> u8 {
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in load8");

        return match region {
            MemoryRegion::BIOS => utils::load8(&self.bios.data, offset),
            MemoryRegion::RAM => utils::load8(&self.ram.data, offset),
            MemoryRegion::Expansion1 => {
                trace!("Unhandled load8 at Expansion1 range.");
                0xff
            }
            _ => panic!(
                "Unhandled load8 at address 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        };
    }

    pub fn load16(&self, addr: u32) -> u16 {
        expect_align(addr, 2);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in load16");

        return match region {
            MemoryRegion::BIOS => utils::load16(&self.bios.data, offset),
            MemoryRegion::RAM => utils::load16(&self.ram.data, offset),
            MemoryRegion::IO | MemoryRegion::SPU => {
                trace!("Unhandled load16 at {:?} range.", region);
                0x0
            }
            _ => panic!(
                "Unhandled load16 at address 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        };
    }

    pub fn load32(&self, addr: u32) -> u32 {
        expect_align(addr, 4);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in load32");

        return match region {
            MemoryRegion::BIOS => utils::load32(&self.bios.data, offset),
            MemoryRegion::RAM => utils::load32(&self.ram.data, offset),
            MemoryRegion::IRQControl | MemoryRegion::Timers => {
                debug!("Ignoring read from {:?} range: 0x{:08X}", region, offset);
                0
            }
            MemoryRegion::Expansion1 | MemoryRegion::IO => {
                trace!("Unhandled load32 at {:?} range.", region);
                0xff
            }
            _ => panic!(
                "Unhandled load32 at address 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        };
    }

    pub fn store8(&mut self, addr: u32, value: u8) {
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in store8");

        match region {
            MemoryRegion::RAM => utils::store8(&mut self.ram.data, offset, value),
            MemoryRegion::Expansion1 | MemoryRegion::Expansion2 => {
                debug!("Unhandled write to {:?} at offset 0x{:08X}", region, offset);
            }
            _ => panic!(
                "Unhandled store8 at address 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        }
    }

    pub fn store16(&mut self, addr: u32, value: u16) {
        expect_align(addr, 2);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in store16");

        match region {
            MemoryRegion::RAM => utils::store16(&mut self.ram.data, offset, value),
            MemoryRegion::Timers | MemoryRegion::SPU => {
                debug!("Unhandled write to {:?} register: 0x{:08X}", region, offset);
            }
            _ => panic!(
                "Unhandled store16 at address 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        }
    }

    pub fn store32(&mut self, addr: u32, value: u32) {
        expect_align(addr, 4);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in store32");

        match region {
            MemoryRegion::RAM => utils::store32(&mut self.ram.data, offset, value),
            MemoryRegion::BIOS => {
                panic!("Illegal write to BIOS memory");
            }
            MemoryRegion::MemControl => {
                check_memcontrol(offset, value);
            }
            MemoryRegion::IRQControl
            | MemoryRegion::RAMSize
            | MemoryRegion::CacheControl
            | MemoryRegion::Timers
            | MemoryRegion::IO => {
                debug!("Ignoring write to {:?} range: 0x{:08X}", region, offset);
            }
            _ => panic!(
                "Unhandled store32 at address 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        }
    }
}

fn expect_align(addr: u32, align: u32) {
    if addr % align != 0 {
        panic!(
            "Unaligned memory access for address 0x{:08X}... expected alignment of {}",
            addr, align
        );
    }
}

fn check_memcontrol(offset: u32, value: u32) {
    match (offset, value) {
        (0, 0x1f000000) => return,
        (0, _) => panic!("Bad expansion 1 base address: 0x{:08X}", value),
        (4, 0x1f802000) => return,
        (4, _) => panic!("Bad expansion 1 base address: 0x{:08X}", value),
        _ => debug!("Unhandled write to MEMCONTROL register."),
    }
}
