use crate::map::RAM_SIZE;

pub struct RAM {
    pub data: Vec<u8>,
}

impl RAM {
    pub fn new() -> RAM {
        let data = vec![0x69; RAM_SIZE as usize];
        RAM { data }
    }
}
