// It's bussin my g
use log::debug;

use crate::bios::BIOS;
use crate::map;
use crate::map::MemoryRegion;
use crate::ram::RAM;
use crate::utils;
use std::string::String;

pub struct Bus {
    bios: BIOS,
    ram: RAM,
}

impl Bus {
    pub fn new(bios: BIOS, ram: RAM) -> Bus {
        Bus { bios, ram }
    }

    pub fn load8(&self, addr: u32) -> Result<u8, String> {
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in load8");

        return match region {
            MemoryRegion::BIOS => Ok(utils::load8(&self.bios.data, offset)),
            MemoryRegion::RAM => Ok(utils::load8(&self.ram.data, offset)),
            MemoryRegion::Expansion1 => {
                trace!("Unhandled load8 at Expansion1 range.");
                Ok(0xff)
            }
            _ => Err(format!("Unhandled load8 @ 0x{:08X} (MemoryRegion::{:?})", addr, region)),
        };
    }

    pub fn load16(&self, addr: u32) -> Result<u16, String> {
        expect_align(addr, 2);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in load16");

        return match region {
            MemoryRegion::BIOS => Ok(utils::load16(&self.bios.data, offset)),
            MemoryRegion::RAM => Ok(utils::load16(&self.ram.data, offset)),
            MemoryRegion::IO | MemoryRegion::SPU => {
                trace!("Unhandled load16 at {:?} range.", region);
                Ok(0x0)
            }
            _ => Err(format!("Unhandled load16 @ 0x{:08X} (MemoryRegion::{:?})", addr, region)),
        };
    }

    pub fn load32(&self, addr: u32) -> Result<u32, String> {
        expect_align(addr, 4);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in load32");

        return match region {
            MemoryRegion::BIOS => Ok(utils::load32(&self.bios.data, offset)),
            MemoryRegion::RAM => Ok(utils::load32(&self.ram.data, offset)),
            MemoryRegion::IRQControl | MemoryRegion::Timers => {
                debug!("Ignoring read from {:?} range: 0x{:08X}", region, offset);
                Ok(0)
            }
            MemoryRegion::Expansion1 | MemoryRegion::IO => {
                trace!("Unhandled load32 at {:?} range.", region);
                Ok(0xff)
            }
            _ => Err(format!("Unhandled load32 @ 0x{:08X} (MemoryRegion::{:?})", addr, region)),
        };
    }

    pub fn store8(&mut self, addr: u32, value: u8) -> Result<(), String> {
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in store8");

        match region {
            MemoryRegion::RAM => utils::store8(&mut self.ram.data, offset, value),
            MemoryRegion::Expansion1 | MemoryRegion::Expansion2 => {
                debug!("Unhandled write to {:?} at offset 0x{:08X}", region, offset);
            }
            _ => return Err(format!("Unhandled store8 @ 0x{:08X} = {} (MemoryRegion::{:?})", addr, value, region)),
        }
        Ok(())
    }

    pub fn store16(&mut self, addr: u32, value: u16) -> Result<(), String> {
        expect_align(addr, 2);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in store16");

        match region {
            MemoryRegion::RAM => utils::store16(&mut self.ram.data, offset, value),
            MemoryRegion::Timers | MemoryRegion::SPU => {
                debug!("Unhandled write to {:?} register: 0x{:08X}", region, offset);
            }
            _ => return Err(format!("Unhandled store16 @ 0x{:08X} = {} (MemoryRegion::{:?})", addr, value, region)),
        }
        Ok(())
    }

    pub fn store32(&mut self, addr: u32, value: u32) -> Result<(), String> {
        expect_align(addr, 4);
        let (region, offset) = map::find_region(addr).expect("Unknown memory region in store32");

        match region {
            MemoryRegion::RAM => utils::store32(&mut self.ram.data, offset, value),
            MemoryRegion::BIOS => return Err(String::from("Illegal write to BIOS memory")),
            MemoryRegion::MemControl => check_memcontrol(offset, value)?,
            MemoryRegion::IRQControl
            | MemoryRegion::RAMSize
            | MemoryRegion::CacheControl
            | MemoryRegion::Timers
            | MemoryRegion::IO => {
                debug!("Ignoring write to {:?} range: 0x{:08X}", region, offset);
            }
            _ => return Err(format!("Unhandled store32 @ 0x{:08X} = {} (MemoryRegion::{:?})", addr, value, region)),
        }
        Ok(())
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

fn check_memcontrol(offset: u32, value: u32) -> Result<(), String> {
    return match (offset, value) {
        (0, 0x1f000000) => Ok(()),
        (0, _) => Err(format!("Bad expansion 1 base address: 0x{:08X}", value)),
        (4, 0x1f802000) => Ok(()),
        (4, _) => Err(format!("Bad expansion 2 base address: 0x{:08X}", value)),
        _ => {
            debug!("Unhandled write to MEMCONTROL register.");
            Ok(())
        }
    }
}
