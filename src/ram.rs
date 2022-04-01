use crate::range::Range;

pub const RAM_SIZE: usize = 2 * 1024 * 1024;
const RANGE: Range = Range(0x00000000, RAM_SIZE as u32);


pub struct RAM {
    data: Vec<u8>
}

impl RAM {
    pub fn new() -> RAM {
        let data = vec![0x69; RAM_SIZE];

        RAM { data }
    }

    pub fn contains(addr: u32) -> Option<u32> {
        RANGE.contains(addr)
    }

    pub fn load8(&self, offset: u32) -> u8 {
        self.data[offset as usize]
    }

    pub fn load16(&self, offset: u32) -> u16 {
        let offset = offset as usize;

        let mut result: u16 = 0x0;
        for byte_offset in 0..2 {
            result = result | (self.data[offset + byte_offset] as u16) << (byte_offset * 8);
        }
        result
    }

    pub fn load32(&self, offset: u32) -> u32 {
        let offset = offset as usize;

        let mut result = 0x0;
        for byte_offset in 0..4 {
            result = result | (self.data[offset + byte_offset] as u32) << (byte_offset * 8);
        }
        result
    }

    pub fn store8(&mut self, offset: u32, value: u8) {
        self.data[offset as usize] = value;
    }

    pub fn store16(&mut self, offset: u32, value: u16) {
        let offset = offset as usize;

        let mut value = value;

        for byte_offset in 0..2 {
            self.data[offset + byte_offset] = value as u8;
            value = value >> 8;
        }
    }

    pub fn store32(&mut self, offset: u32, value: u32) {
        let offset = offset as usize;

        let mut value = value;

        for byte_offset in 0..4 {
            self.data[offset + byte_offset] = value as u8;
            value = value >> 8;
        }
    }

}
