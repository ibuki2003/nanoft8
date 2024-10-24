// bitset that can store up to 96 bits
// NOTE: inner value is bit-reversed (i.e. 1<<31 represents the first bit in the message)
#[derive(Debug, Clone, Default)]
pub struct Bitset([u32; Self::LEN]);

impl Bitset {
    pub const SIZE: usize = 96;
    pub const LEN: usize = (Self::SIZE + 31) / 32;

    #[inline]
    pub fn slice(&self, start: usize, size: usize) -> u32 {
        debug_assert!(size <= 32);
        debug_assert!(start + size <= Self::SIZE);
        let start_word = start / 32;
        let start_bit = start % 32;
        let mut v = self.0[start_word] << start_bit;
        if start_bit + size > 32 {
            v |= self.0[start_word + 1] >> (32 - start_bit);
        }
        v >> (32 - size)
    }

    #[inline]
    pub fn set_slice(&mut self, start: usize, size: usize, value: u32) {
        let value = value & ((1 << size) - 1);
        debug_assert!(size <= 32);
        debug_assert!(start + size <= Self::SIZE);
        let start_word = start / 32;
        let start_bit = start % 32;

        if start_bit + size <= 32 {
            set_range(&mut self.0[start_word], start_bit, size, value);
        } else {
            let len1 = 32 - start_bit;
            let len2 = size - len1;
            set_range(
                &mut self.0[start_word],
                start_bit,
                len1,
                value >> (size - len1),
            );
            set_range(
                &mut self.0[start_word + 1],
                0,
                len2,
                value & ((1 << len2) - 1),
            );
        }
    }

    #[inline]
    pub fn slice_u64(&self, start: usize, size: usize) -> u64 {
        debug_assert!(size <= 64);
        debug_assert!(start + size <= Self::SIZE);
        let start_word = start / 32;
        let start_bit = start % 32;

        let mut v = (self.0[start_word] as u64) << (32 + start_bit);
        if start_bit + size > 32 {
            v |= (self.0[start_word + 1] as u64) << start_bit;
        }
        if start_bit + size > 64 {
            v |= (self.0[start_word + 2] as u64) >> (32 - start_bit);
        }
        v >> (64 - size)
    }

    #[inline]
    pub fn get(&self, i: usize) -> bool {
        debug_assert!(i < Self::SIZE);
        let word = i / 32;
        let bit = 31 - (i % 32);
        self.0[word] & (1 << bit) != 0
    }

    #[inline]
    pub fn set(&mut self, i: usize, value: bool) {
        debug_assert!(i < Self::SIZE);
        let word = i / 32;
        let bit = 31 - (i % 32);
        if value {
            self.0[word] |= 1 << bit;
        } else {
            self.0[word] &= !(1 << bit);
        }
    }
}

#[inline]
fn set_range(v: &mut u32, start: usize, size: usize, value: u32) {
    debug_assert!(size <= 32);
    debug_assert!(start + size <= 32);
    debug_assert!(value < 1 << size);

    let value = value & ((1 << size) - 1);
    let mask = !(((1 << size) - 1) << (32 - size - start));
    *v &= mask;
    *v |= value << (32 - size - start);
}

#[cfg(not(feature = "no_std"))]
impl std::fmt::Display for Bitset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..Self::SIZE {
            write!(f, "{}", if self.get(i) { "1" } else { "0" })?;
        }
        Ok(())
    }
}
