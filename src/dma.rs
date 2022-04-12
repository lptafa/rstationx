pub struct DMA {
    control: u32,
}

impl DMA {
    pub fn new() -> DMA {
        // From Nocash PSX Spec
        DMA {
            control: 0x07654321,
        }
    }

    pub fn control(&self) -> u32 {
        self.control
    }

    pub fn set_control(&mut self, value: u32) {
        self.control = value;
    }
}
