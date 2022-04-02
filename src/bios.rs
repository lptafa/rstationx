use crate::map::BIOS_SIZE;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use log::info;

pub struct BIOS {
    #[allow(dead_code)]
    data: Vec<u8>,
}

impl BIOS {
    pub fn new(path: &Path) -> std::io::Result<BIOS> {
        info!("Reading BIOS from {}.", path.display());
        let file = File::open(path)?;
        let mut data = Vec::new();

        file.take(BIOS_SIZE as u64).read_to_end(&mut data)?;

        if data.len() == BIOS_SIZE as usize {
            Ok(BIOS { data })
        } else {
            Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Invalid BIOS file.",
            ))
        }
    }

    pub fn load8(&self, offset: u32) -> u8 {
        self.data[offset as usize]
    }

    pub fn load16(&self, offset: u32) -> u16 {
        let offset = offset as usize;

        let mut result = 0x0;
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
}
