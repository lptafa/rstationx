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

    pub fn load<T: TryFrom<u32>>(&self, addr: u32) -> Result<T, String> {
        expect_align(addr, std::mem::size_of::<T>() as u32)?;
        let (region, offset) = map::find_region(addr)?;

        return match region {
            MemoryRegion::BIOS => Ok(utils::load::<T>(&self.bios.data, offset)),
            MemoryRegion::RAM => Ok(utils::load::<T>(&self.ram.data, offset)),
            MemoryRegion::IRQControl
            | MemoryRegion::Timers
            | MemoryRegion::IO
            | MemoryRegion::SPU => {
                trace!("Unhandled load::<u32> at {:?} range.", region);
                Ok(utils::to_t(0))
            }
            MemoryRegion::Expansion1 => {
                trace!("Unexpected load::<u32> at {:?} range.", region);
                Ok(utils::to_t(0xff))
            }
            _ => Err(format!(
                "Unhandled load::<u32> @ 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            )),
        };
    }

    pub fn store<T: Into<u32>>(&mut self, addr: u32, value: T) -> Result<(), String> {
        expect_align(addr, std::mem::size_of::<T>() as u32)?;
        let (region, offset) = map::find_region(addr)?;

        match region {
            MemoryRegion::RAM => utils::store::<T>(&mut self.ram.data, offset, value),
            MemoryRegion::BIOS => return Err(String::from("Illegal write to BIOS memory")),
            MemoryRegion::MemControl => {
                let value = value.into();
                return match (offset, value) {
                    (0, 0x1f000000) => Ok(()),
                    (0, _) => Err(format!("Bad expansion 1 base address")),
                    (4, 0x1f802000) => Ok(()),
                    (4, _) => Err(format!("Bad expansion 2 base address")),
                    _ => {
                        debug!("Unhandled write to MEMCONTROL register.");
                        Ok(())
                    }
                }
            }
            MemoryRegion::IRQControl
            | MemoryRegion::Expansion1
            | MemoryRegion::Expansion2
            | MemoryRegion::RAMSize
            | MemoryRegion::CacheControl
            | MemoryRegion::SPU
            | MemoryRegion::Timers
            | MemoryRegion::IO => {
                debug!("Ignoring write to {:?} range: 0x{:08X}", region, offset);
            }
        }
        Ok(())
    }
}

fn expect_align(addr: u32, align: u32) -> Result<(), String>{
    if addr % align != 0 {
        Err(
            format!("Unaligned memory access for address 0x{:08X}... expected alignment of {}",
            addr, align)
        )
    } else {
        Ok(())
    }
}
