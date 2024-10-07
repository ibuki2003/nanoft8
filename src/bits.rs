// bitset that can store up to 96 bits
// NOTE: inner value is bit-reversed (i.e. 1<<31 represents the first bit in the message)
#[derive(Debug, Clone, Default)]
pub struct Bitset([u32; Self::LEN]);

impl Bitset {
    pub const SIZE: usize = 96;
    pub const LEN: usize = (Self::SIZE + 31) / 32;

    #[inline]
    pub fn slice(&self, start: usize, size: usize) -> u32 {
        assert!(size <= 32);
        assert!(start + size <= Self::SIZE);
        let start_word = start / 32;
        let start_bit = start % 32;
        let mut v = self.0[start_word] << start_bit;
        if start_bit + size > 32 {
            v |= self.0[start_word + 1] >> (32 - start_bit);
        }
        v >> (32 - size)
    }

    #[inline]
    pub fn slice_u64(&self, start: usize, size: usize) -> u64 {
        let mut v = (self.slice(start, size.min(32)) as u64) << 32;
        if size > 32 {
            v |= self.slice(start + 32, size - 32) as u64;
        }
        v >> (64 - size)
    }

    #[inline]
    pub fn get(&self, i: usize) -> bool {
        assert!(i < Self::SIZE);
        let word = i / 32;
        let bit = 31 - (i % 32);
        self.0[word] & (1 << bit) != 0
    }

    #[inline]
    pub fn set(&mut self, i: usize, value: bool) {
        assert!(i < Self::SIZE);
        let word = i / 32;
        let bit = 31 - (i % 32);
        if value {
            self.0[word] |= 1 << bit;
        } else {
            self.0[word] &= !(1 << bit);
        }
    }

}
