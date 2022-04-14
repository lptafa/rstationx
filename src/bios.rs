use crate::memory::BIOS_SIZE;
use crate::utils;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use log::info;

pub struct BIOS {
    pub data: Vec<u8>,
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

    #[inline]
    pub fn load<T: TryFrom<u32>>(&self, addr: u32) -> T {
        utils::load(&self.data, addr)
    }
}
