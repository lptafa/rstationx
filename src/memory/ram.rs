use super::map::RAM_SIZE;
use crate::utils;

pub struct RAM {
    pub data: Vec<u8>,
}

impl RAM {
    pub fn new() -> RAM {
        let data = vec![0x69; RAM_SIZE as usize];
        RAM { data }
    }

    #[inline]
    pub fn load<T: TryFrom<u32>>(&self, addr: u32) -> T {
        utils::load(&self.data, addr)
    }

    #[inline]
    pub fn store<T: Into<u32>>(&mut self, addr: u32, value: T) {
        utils::store(&mut self.data, addr, value)
    }
}
