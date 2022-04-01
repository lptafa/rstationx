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
