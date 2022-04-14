use super::channel::Channel;

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
