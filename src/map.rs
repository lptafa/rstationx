// Global sizes and offsets for regions

pub const BIOS_SIZE: u32 = 512 * 1024;
pub const BIOS_START: u32 = 0x1fc00000;

pub const RAM_SIZE: u32 = 2 * 1024 * 1024;
pub const RAM_START: u32 = 0x00000000;

// Region masking

const REGION_MASK: [u32; 8] = [
    0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, // KUSEG: 2048MB
    0x7fffffff, // KSEG0:  512MB
    0x1fffffff, // KSEG1:  512MB
    0xffffffff, 0xffffffff, // KSEG2: 1024MB
];

fn mask_region(addr: u32) -> u32 {
    let index = (addr >> 29) as usize;
    addr & REGION_MASK[index]
}

// Regions defined below

#[derive(Debug, Clone, Copy)]
pub struct Range(pub u32, pub u32);

impl Range {
    pub fn contains(self, addr: u32) -> Option<u32> {
        let Range(start, length) = self;
        if addr >= start && addr < start + length {
            Some(addr - start)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryRegion {
    RAM,
    BIOS,
    MemControl,
    RAMSize,
    Expansion1,
    Expansion2,
    SPU,
    IO,
    IRQControl,
    Timers,
    CacheControl,
}

// Note: Increment the array size if you add a new region.
// Note: Put the most frequently accessed regions first, for performance.
const ALL_REGIONS: [(MemoryRegion, Range); 11] = [
    (MemoryRegion::RAM, Range(RAM_START, RAM_SIZE)),
    (MemoryRegion::BIOS, Range(BIOS_START, BIOS_SIZE)),
    (MemoryRegion::Expansion1, Range(0x1f000000, 8 * 1024 * 1024)),
    (MemoryRegion::MemControl, Range(0x1f801000, 36)),
    (MemoryRegion::RAMSize, Range(0x1f801060, 4)),
    (MemoryRegion::CacheControl, Range(0xfffe0130, 4)),
    (MemoryRegion::SPU, Range(0x1f801c00, 640)),
    (MemoryRegion::Expansion2, Range(0x1f802000, 66)),
    (MemoryRegion::IRQControl, Range(0x1f801070, 8)),
    (MemoryRegion::Timers, Range(0x1f801100, 0x30)),
    // FIXME: This is not right
    (MemoryRegion::IO, Range(0x1f801000, 8 * 1024)),
];

pub fn find_region(addr: u32) -> Option<(MemoryRegion, u32)> {
    let addr = mask_region(addr);
    for (region, range) in ALL_REGIONS.iter() {
        if let Some(offset) = range.contains(addr) {
            return Some((*region, offset));
        }
    }
    return None;
}
