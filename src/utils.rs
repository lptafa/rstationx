// This module contains some utilities that I don't know where else to put for now.
use std::vec::Vec;

pub fn load8(buf: &Vec<u8>, offset: u32) -> u8 {
    buf[offset as usize]
}

pub fn load16(buf: &Vec<u8>, offset: u32) -> u16 {
    let offset = offset as usize;

    let mut result: u16 = 0x0;
    for byte_offset in 0..2 {
        result = result | (buf[offset + byte_offset] as u16) << (byte_offset * 8);
    }
    result
}

pub fn load32(buf: &Vec<u8>, offset: u32) -> u32 {
    let offset = offset as usize;

    let mut result = 0x0;
    for byte_offset in 0..4 {
        result = result | (buf[offset + byte_offset] as u32) << (byte_offset * 8);
    }
    result
}

pub fn store8(buf: &mut Vec<u8>, offset: u32, value: u8) {
    buf[offset as usize] = value;
}

pub fn store16(buf: &mut Vec<u8>, offset: u32, value: u16) {
    let offset = offset as usize;

    let mut value = value;

    for byte_offset in 0..2 {
        buf[offset + byte_offset] = value as u8;
        value = value >> 8;
    }
}

pub fn store32(buf: &mut Vec<u8>, offset: u32, value: u32) {
    let offset = offset as usize;

    let mut value = value;

    for byte_offset in 0..4 {
        buf[offset + byte_offset] = value as u8;
        value = value >> 8;
    }
}