// NOTE: inner value is bit-reversed (i.e. 1<<31 represents the first bit (i.e. index 0))
#[derive(Debug, Clone, Copy)]
pub struct Bitset<const SIZE: usize>(pub [u32; num_words::<SIZE>()])
where
    [u32; num_words::<SIZE>()]: Sized;

pub const fn num_words<const SIZE: usize>() -> usize {
    (SIZE + 31) / 32
}

impl<const SIZE: usize> Default for Bitset<SIZE>
where
    [u32; num_words::<SIZE>()]: Sized,
{
    fn default() -> Self {
        Self([0; num_words::<SIZE>()])
    }
}

impl<const SIZE: usize> Bitset<SIZE>
where
    [u32; num_words::<SIZE>()]: Sized,
{
    pub const SIZE: usize = SIZE;
    pub const LEN: usize = num_words::<SIZE>();

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
        debug_assert!(size <= 32);
        debug_assert!(start + size <= Self::SIZE);
        let value = if size == 32 {
            value
        } else {
            value & ((1 << size) - 1)
        };
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

    pub fn with_size<const NEW_SIZE: usize>(self) -> Bitset<NEW_SIZE>
    where
        [u32; num_words::<NEW_SIZE>()]: Sized,
    {
        let mut new = Bitset::<NEW_SIZE>::default();
        let min = Self::LEN.min(num_words::<NEW_SIZE>());
        new.0[..min].copy_from_slice(&self.0[..min]);
        if NEW_SIZE < Self::SIZE {
            if let Some(v) = new.0.last_mut() {
                let mask = (!0) << (32 - (NEW_SIZE % 32));
                *v &= mask;
            }
        }
        new
    }
}

impl<const SIZE: usize> From<[u32; num_words::<SIZE>()]> for Bitset<SIZE>
where
    [u32; num_words::<SIZE>()]: Sized,
{
    fn from(arr: [u32; num_words::<SIZE>()]) -> Self {
        Self(arr)
    }
}

#[inline]
fn set_range(v: &mut u32, start: usize, size: usize, value: u32) {
    debug_assert!(size <= 32);
    debug_assert!(start + size <= 32);
    if size < 32 {
        debug_assert!(value < 1 << size);
    }

    // let value = value & ((1 << size) - 1);
    let mask = if size == 32 { !0 } else { (1 << size) - 1 };
    let mask = !(mask << (32 - size - start));
    *v &= mask;
    *v |= value << (32 - size - start);
}

#[cfg(not(feature = "no_std"))]
impl<const SIZE: usize> std::fmt::Display for Bitset<SIZE>
where
    [u32; num_words::<SIZE>()]: Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..Self::SIZE {
            write!(f, "{}", if self.get(i) { "1" } else { "0" })?;
        }
        Ok(())
    }
}
