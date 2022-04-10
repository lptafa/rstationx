// It's bussin my g
use log::debug;

use crate::bios::BIOS;
use crate::gpu::GPU;
use crate::map;
use crate::map::MemoryRegion;
use crate::ram::RAM;
use crate::utils;
use std::string::String;

pub struct Bus {
    bios: BIOS,
    gpu: GPU,
    ram: RAM,
}

impl Bus {
    pub fn new(bios: BIOS, ram: RAM, gpu: GPU) -> Bus {
        Bus { bios, ram, gpu }
    }

    pub fn load<T: TryFrom<u32>>(&self, addr: u32) -> Result<T, String> {
        expect_align(addr, std::mem::size_of::<T>() as u32)?;
        let (region, offset) = map::find_region(addr)?;

        return match region {
            MemoryRegion::BIOS => Ok(utils::load::<T>(&self.bios.data, offset)),
            MemoryRegion::RAM => Ok(utils::load::<T>(&self.ram.data, offset)),
            MemoryRegion::IRQControl
            | MemoryRegion::Timers
            | MemoryRegion::DMA
            | MemoryRegion::SPU => {
                trace!("Unhandled load at {:?} range.", region);
                Ok(utils::to_t(0))
            }
            MemoryRegion::Expansion1 => {
                trace!("Unexpected load at {:?} range.", region);
                Ok(utils::to_t(0xff))
            }
            MemoryRegion::GPU => Ok(self.gpu.load(offset)),

            _ => Err(format!(
                "Unhandled load @ 0x{:08X} (MemoryRegion::{:?})",
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
                };
            }
            MemoryRegion::IRQControl
            | MemoryRegion::Expansion1
            | MemoryRegion::Expansion2
            | MemoryRegion::RAMSize
            | MemoryRegion::CacheControl
            | MemoryRegion::SPU
            | MemoryRegion::DMA
            | MemoryRegion::GPU
            | MemoryRegion::Timers => {
                debug!("Ignoring write to {:?} range: 0x{:08X}", region, offset);
            }
            _ => return Err(format!("Unhandled write to 0x{:08x} addr", addr)),
        }
        Ok(())
    }
}

fn expect_align(addr: u32, align: u32) -> Result<(), String> {
    if addr % align != 0 {
        Err(format!(
            "Unaligned memory access for address 0x{:08X}... expected alignment of {}",
            addr, align
        ))
    } else {
        Ok(())
    }
}
