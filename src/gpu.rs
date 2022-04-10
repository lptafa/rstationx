use crate::utils;

pub struct GPU {}

impl GPU {
    pub fn new() -> GPU {
        GPU {}
    }

    pub fn load<T: TryFrom<u32>>(&self, offset: u32) -> T {
        let value: u32 = match offset {
            4 => 0x10000000,
            _ => 0,
        };
        utils::to_t(value)
    }
}
