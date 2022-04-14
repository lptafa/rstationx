/**
 * A DMA Channel
 */
use crate::utils::Error;
use log::debug;
use std::string::String;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    ToDevice = 0,
    FromDevice = 1,
}

#[derive(Debug, Copy, Clone)]
pub enum AddressMode {
    Increment = 0,
    Decrement = 1,
}

#[derive(Debug, Copy, Clone)]
pub enum SyncMode {
    Manual = 0,
    Request = 1,
    LinkedList = 2,
}

#[derive(Debug, Copy, Clone)]
pub struct Channel {
    enable: bool,
    direction: Direction,
    address_mode: AddressMode,
    sync_mode: SyncMode,
    manual_trigger: bool,
    chop: bool,
    chop_dma_size: u8,
    chop_cpu_size: u8,
    dummy: u8,

    base: u32,

    block_count: u16,
    block_size: u16,
}

impl Channel {
    pub fn new() -> Channel {
        Channel {
            enable: false,
            direction: Direction::ToDevice,
            address_mode: AddressMode::Increment,
            sync_mode: SyncMode::Manual,
            manual_trigger: false,
            chop: false,
            chop_dma_size: 0,
            chop_cpu_size: 0,
            dummy: 0,
            base: 0,
            block_count: 0,
            block_size: 0,
        }
    }

    pub fn active(&self) -> bool {
        self.enable
            && match self.sync_mode {
                SyncMode::Manual => self.manual_trigger,
                _ => true,
            }
    }

    pub fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn address_mode(&self) -> AddressMode {
        self.address_mode
    }

    pub fn set_base(&mut self, base: u32) {
        debug!("Writing 0x{:08X} to base register", base);
        self.base = base;
    }

    pub fn block_control(&self) -> u32 {
        self.block_size as u32 | ((self.block_count as u32) << 16)
    }

    pub fn set_block_control(&mut self, block_control: u32) {
        debug!("Writing 0x{:08X} to block control register", block_control);
        self.block_size = block_control as u16;
        self.block_count = (block_control >> 16) as u16;
    }

    pub fn transfer_size(&mut self) -> Result<u32, String> {
        let size = self.block_size as u32;
        let count = self.block_count as u32;

        match self.sync_mode {
            SyncMode::Manual => Ok(size),
            SyncMode::Request => Ok(count * size),
            SyncMode::LinkedList => Error!("No linked list mode implemented for DMA channel."),
        }
    }

    pub fn set_finished(&mut self) {
        self.enable = false;
        self.manual_trigger = false;
        debug!("Finished DMA channel");
    }

    pub fn control(&self) -> u32 {
        return (self.direction as u32) << 0
            | (self.address_mode as u32) << 1
            | (self.chop as u32) << 8
            | (self.sync_mode as u32) << 9
            | (self.chop_dma_size as u32) << 16
            | (self.chop_cpu_size as u32) << 20
            | (self.enable as u32) << 24
            | (self.manual_trigger as u32) << 28
            | (self.dummy as u32) << 29;
    }

    pub fn set_control(&mut self, value: u32) -> Result<(), String> {
        debug!("Writing 0x{:08X} to control register", value);
        match value & 0b1 {
            0 => self.direction = Direction::ToDevice,
            1 => self.direction = Direction::FromDevice,
            _ => unreachable!(),
        }

        match (value >> 1) & 0b1 {
            0 => self.address_mode = AddressMode::Increment,
            1 => self.address_mode = AddressMode::Decrement,
            _ => unreachable!(),
        }

        let sync_mode = (value >> 9) & 0b11;
        match sync_mode {
            0 => self.sync_mode = SyncMode::Manual,
            1 => self.sync_mode = SyncMode::Request,
            2 => self.sync_mode = SyncMode::LinkedList,
            _ => {
                return Error!(
                    "Invalid sync mode {} in set control for DMA register",
                    sync_mode
                )
            }
        }

        self.chop = (value & (1 << 8)) != 0;
        self.manual_trigger = (value & (1 << 28)) != 0;
        self.enable = (value & (1 << 24)) != 0;

        self.chop_dma_size = ((value >> 16) & 0b111) as u8;
        self.chop_cpu_size = ((value >> 20) & 0b111) as u8;

        self.dummy = ((value >> 29) & 0b11) as u8;

        Ok(())
    }
}
