use super::chars::Chars;
use crate::Bitset;
use core::hint::assert_unchecked;

pub struct F71(pub Bitset);

impl F71 {
    pub fn from_string(str: &mut [u8]) -> Option<Self> {
        assert!(str.len() <= 13);
        let mut arr = [0u32; 5];

        for c in str.iter() {
            arr[0] = arr[0].wrapping_mul(42);
            for i in 1..5 {
                let v = arr[i].wrapping_mul(42);
                arr[i - 1] = arr[i - 1].wrapping_add(v >> 16);
                arr[i] = v & ((1 << 16) - 1);
            }
            arr[4] = arr[4].wrapping_add((Chars::Full.find(*c)? as u32) << 9);
        }
        for i in (1..5).rev() {
            arr[i - 1] = arr[i - 1].wrapping_add(arr[i] >> 16);
            arr[i] &= (1 << 16) - 1;
        }

        let mut bs = Bitset::default();
        bs.0[0] = arr[0] << 16 | arr[1];
        bs.0[1] = arr[2] << 16 | arr[3];
        bs.0[2] = arr[4] << 16;

        Some(Self(bs))
    }

    pub fn write_str(&self, str: &mut [u8]) -> Option<usize> {
        if str.len() < 13 {
            return None;
        }
        let str = &mut str[..13];
        let mut v = self.0 .0;

        v[2] = v[2] >> 25 | v[1] << 7;
        v[1] = v[1] >> 25 | v[0] << 7;
        v[0] >>= 25;

        for c in str.iter_mut().rev() {
            let rem = v.iter_mut().fold(0, |rem, val| {
                let rem = rem << 32 | *val as u64;
                unsafe {
                    assert_unchecked(rem / 42 < (1 << 32));
                }
                *val = (rem / 42) as u32;
                rem % 42
            }) as u8;
            *c = Chars::Full.get(rem);
        }

        Some(13)
    }
}

#[cfg(not(feature = "no_std"))]
impl core::fmt::Display for F71 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut str = [0u8; 13];
        self.write_str(&mut str);
        f.write_str(core::str::from_utf8(&str).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freetext() {
        let testcases = &[
            (b"             ", [0, 0, 0]),
            (b"            0", [0, 0x0, 0x02000000]),
            (b"0000000000000", [0x358a849, 0x93e71807, 0xce000000]),
            (b"ZZZZZZZZZZZZZ", [0x7877aa58, 0xcc7f6118, 0xf8000000]),
            (b"?????????????", [0x8932F3C8, 0xB002D93F, 0xFE000000]),
        ];
        for (expected, bits) in testcases {
            let mut str = [b' '; 13];
            F71(Bitset(*bits)).write_str(&mut str).unwrap();
            assert_eq!(&str, *expected);

            let f = F71::from_string(&mut str).unwrap();
            assert_eq!(f.0 .0, *bits);
        }
    }
}
