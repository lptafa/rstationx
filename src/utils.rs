// This module contains some utilities that I don't know where else to put for now.
use std::vec::Vec;

pub fn load<T: TryFrom<u32>>(buf: &Vec<u8>, offset: u32) -> T {
    let offset = offset as usize;

    let mut result = 0x0;
    for byte_offset in 0..std::mem::size_of::<T>() {
        result = result | (buf[offset + byte_offset] as u32) << (byte_offset * 8);
    }
    to_t(result)
}

pub fn store<T: Into<u32>>(buf: &mut Vec<u8>, offset: u32, value: T) {
    let offset = offset as usize;

    let mut value = value.into();

    for byte_offset in 0..std::mem::size_of::<T>() {
        buf[offset + byte_offset] = value as u8;
        value = value >> 8;
    }
}

pub fn to_t<T: TryFrom<u32>>(i: u32) -> T {
    T::try_from(i).unwrap_or_else(|_| panic!("Invalid integer."))
}
