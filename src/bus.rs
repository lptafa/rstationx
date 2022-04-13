// It's bussin my g

use crate::bios::BIOS;
use crate::dma::{Port, SyncMode, DMA, AddressMode};
use crate::gpu::GPU;
use crate::map;
use crate::map::MemoryRegion;
use crate::ram::RAM;
use crate::utils;
use crate::utils::Error;
use std::string::String;

pub struct Bus {
    bios: BIOS,
    gpu: GPU,
    ram: RAM,
    dma: DMA,
}

impl Bus {
    pub fn new(bios: BIOS, ram: RAM, gpu: GPU) -> Bus {
        let dma = DMA::new();
        Bus {
            bios,
            ram,
            gpu,
            dma,
        }
    }

    pub fn load<T: TryFrom<u32>>(&self, addr: u32) -> Result<T, String> {
        expect_align(addr, std::mem::size_of::<T>() as u32)?;
        let (region, offset) = map::find_region(addr)?;

        return match region {
            MemoryRegion::BIOS => Ok(utils::load::<T>(&self.bios.data, offset)),
            MemoryRegion::RAM => Ok(utils::load::<T>(&self.ram.data, offset)),
            // FIXME: This is ugly, maybe find a nice way to convert the error from
            //        T.try_into() into our own error type (String)?
            MemoryRegion::DMA => Ok(utils::to_t(self.dma_register(offset)?)),
            MemoryRegion::IRQControl | MemoryRegion::Timers | MemoryRegion::SPU => {
                trace!("Unhandled load at {:?} range.", region);
                Ok(utils::to_t(0))
            }
            MemoryRegion::Expansion1 => {
                trace!("Unexpected load at {:?} range.", region);
                Ok(utils::to_t(0xff))
            }
            MemoryRegion::GPU => Ok(self.gpu.load(offset)),

            _ => Error!(
                "Unhandled load @ 0x{:08X} (MemoryRegion::{:?})",
                addr, region
            ),
        };
    }

    pub fn store<T: Into<u32>>(&mut self, addr: u32, value: T) -> Result<(), String> {
        expect_align(addr, std::mem::size_of::<T>() as u32)?;
        let (region, offset) = map::find_region(addr)?;

        match region {
            MemoryRegion::RAM => utils::store::<T>(&mut self.ram.data, offset, value),
            MemoryRegion::BIOS => return Error!("Illegal write to BIOS memory"),
            MemoryRegion::DMA => return self.set_dma_register(offset, value.into()),
            MemoryRegion::MemControl => {
                let value = value.into();
                return match (offset, value) {
                    (0, 0x1f000000) => Ok(()),
                    (0, _) => Error!("Bad expansion 1 base address"),
                    (4, 0x1f802000) => Ok(()),
                    (4, _) => Error!("Bad expansion 2 base address"),
                    _ => {
                        trace!("Unhandled write to MEMCONTROL register.");
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
            | MemoryRegion::GPU
            | MemoryRegion::Timers => {
                trace!("Ignoring write to {:?} range: 0x{:08X}", region, offset);
            }
        }
        Ok(())
    }

    fn dma_register(&self, offset: u32) -> Result<u32, String> {
        let (major, minor) = (offset >> 4, offset & 0b1111);
        match major {
            // Channels
            0x0..=0x6 => {
                let port = Port::from_index(major).unwrap();
                let channel = self.dma.channel(port);
                match minor {
                    0x0 => Ok(channel.base()),
                    0x4 => Ok(channel.block_control()),
                    0x8 => Ok(channel.control()),
                    _ => Error!(
                        "Unsupported read from minor register {} for channel {}",
                        minor, major
                    ),
                }
            }
            // Common DMA registers
            0x7 => match minor {
                0x0 => Ok(self.dma.control()),
                0x4 => Ok(self.dma.interrupt()),
                _ => Error!(
                    "Unsupported read from minor register {} for major 0x7",
                    minor
                ),
            },
            _ => Error!("Unhandled DMA register read: 0x{:08X}", offset),
        }
    }

    fn set_dma_register(&mut self, offset: u32, value: u32) -> Result<(), String> {
        let (major, minor) = (offset >> 4, offset & 0b1111);
        let active_port = match major {
            // Channels
            0x0..=0x6 => {
                let port = Port::from_index(major).unwrap();
                let channel = self.dma.channel_mut(port);
                match minor {
                    0x0 => channel.set_base(value),
                    0x4 => channel.set_block_control(value),
                    0x8 => channel.set_control(value)?,  // Might fail, so we propagate the error
                    _ => return Error!("Unsupported write to minor register 0x{:02x} for channel 0x{:02x}, value=0x{:08x}", minor, major, value),
                }

                if channel.active() {
                    Some(port)
                } else {
                    None
                }
            }
            // Common DMA registers
            0x7 => {
                match minor {
                    0x0 => self.dma.set_control(value),
                    0x4 => self.dma.set_interrupt(value),
                    _ => return Error!("Unsupported write to minor register 0x{:02x} for channel 0x{:02x}, value=0x{:08x}", minor, major, value),
                }
                None
            }
            _ => {
                return Error!(
                    "Unhandled DMA register write: 0x{:04X}, value=0x{:08X}",
                    offset, value
                )
            }
        };

        if let Some(port) = active_port {
            self.do_dma(port)
        } else {
            Ok(())
        }
    }

    fn do_dma(&mut self, port: Port) -> Result<(), String> {
        match self.dma.channel(port).sync_mode() {
            SyncMode::LinkedList => self.do_dma_linked_list(port),
            _ => self.do_dma_block(port),
        }
    }

    fn do_dma_block(&mut self, port: Port) -> Result<(), String> {
        let channel = self.dma.channel_mut(port);
        let increment = match channel.address_mode() {
            AddressMode::Increment =>  4,
            AddressMode::Decrement => -4,
        };



        Error!("DMA Block not implemented, port: {:?}", port)
    }

    fn do_dma_linked_list(&mut self, port: Port) -> Result<(), String> {
        Error!("DMA LinkedList not implemented, port: {:?}", port)
    }
}

fn expect_align(addr: u32, align: u32) -> Result<(), String> {
    if addr % align != 0 {
        Error!(
            "Unaligned memory access for address 0x{:08X}... expected alignment of {}",
            addr, align
        )
    } else {
        Ok(())
    }
}
