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
    fn new() -> Channel {
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

#[derive(Debug)]
pub struct DMA {
    control: u32,

    // Interrupt register (being split up into components)
    irq_enable: bool,
    channel_irq_enable: u8,
    channel_irq_flags: u8,
    force_irq: bool,
    irq_dummy: u8,

    channels: [Channel; 7],
}

// NOTE: I don't think having this Port abstraction is currently needed, since we never
//       explicitly specify a port in the code. The guide says it'll be useful later so
//       I'm just adding it here while I'm at it.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Port {
    MacroDecoderIn,
    MacroDecoderOut,
    GPU,
    CDRom,
    SPU,
    PIO,
    OTC,
}

impl Port {
    pub fn from_index(index: u32) -> Option<Port> {
        match index {
            0 => Some(Port::MacroDecoderIn),
            1 => Some(Port::MacroDecoderOut),
            2 => Some(Port::GPU),
            3 => Some(Port::CDRom),
            4 => Some(Port::SPU),
            5 => Some(Port::PIO),
            6 => Some(Port::OTC),
            _ => None,
        }
    }
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            // From Nocash PSX Spec
            control: 0x07654321,
            irq_enable: false,
            channel_irq_enable: 0,
            channel_irq_flags: 0,
            force_irq: false,
            irq_dummy: 0,
            channels: [Channel::new(); 7],
        }
    }

    pub fn irq(&self) -> bool {
        let masked = self.channel_irq_enable & self.channel_irq_flags;
        self.force_irq || (self.irq_enable && masked != 0)
    }

    pub fn interrupt(&self) -> u32 {
        return (self.irq_dummy as u32)
            | (self.force_irq as u32) << 15
            | (self.channel_irq_enable as u32) << 16
            | (self.irq_enable as u32) << 23
            | (self.channel_irq_flags as u32) << 24
            | (self.irq() as u32) << 31;
    }

    pub fn set_interrupt(&mut self, val: u32) {
        self.irq_dummy = (val & 0x3f) as u8;
        self.force_irq = (val >> 15) & 1 != 0;
        self.channel_irq_enable = ((val >> 16) & 0x7f) as u8;
        self.irq_enable = (val >> 23) & 1 != 0;

        // Writing 1 to a flag resets it
        let ack = ((val >> 24) & 0x3f) as u8;
        self.channel_irq_flags &= !ack;
    }

    pub fn control(&self) -> u32 {
        self.control
    }

    pub fn set_control(&mut self, value: u32) {
        self.control = value;
    }

    pub fn channel(&self, port: Port) -> &Channel {
        &self.channels[port as usize]
    }

    pub fn channel_mut(&mut self, port: Port) -> &mut Channel {
        &mut self.channels[port as usize]
    }
}
