use core::ops::RangeInclusive;

use super::chars::Chars;

pub struct C28(pub u32);

// standard callsigns
impl C28 {
    pub const VALUE_DE: u32 = 0;
    pub const VALUE_QRZ: u32 = 1;
    pub const VALUE_CQ: u32 = 2;

    pub const DE: Self = Self(Self::VALUE_DE);
    pub const QRZ: Self = Self(Self::VALUE_QRZ);
    pub const CQ: Self = Self(Self::VALUE_CQ);

    const VALUE_CQNUM_RANGE: RangeInclusive<u32> = 3..=1002;
    const VALUE_CQZONE_RANGE: RangeInclusive<u32> = 1003..=(1003 + 27 * 27 * 27 * 27);
    const VALUE_HASH_RANGE: RangeInclusive<u32> = 2063592..=(2063592 + (1 << 20) - 1);
    const VALUE_CALLSIGN_RANGE: RangeInclusive<u32> =
        6257896..=(6257896 + 37 * 36 * 10 * 27 * 27 * 27 - 1);

    const CHARS: [Chars; 6] = [
        Chars::AlnumSpc,
        Chars::Alnum,
        Chars::Numeric,
        Chars::AlphaSpc,
        Chars::AlphaSpc,
        Chars::AlphaSpc,
    ];

    fn normalize_callsign(call: &[u8], idx: &mut [u8]) -> bool {
        if call.len() < 2 || call.len() > 6 {
            return false;
        }
        idx.iter_mut().for_each(|x| *x = 0); // b' '
        'outer: for ofs in 0..=(6 - call.len()) {
            if ofs > 0 {
                idx[ofs - 1] = 0;
            }
            if call.len() == 2 && ofs == 0 {
                continue;
            }

            for (i, &c) in call.iter().enumerate() {
                match Self::CHARS[i + ofs].find(c) {
                    Some(x) => idx[i + ofs] = x,
                    None => continue 'outer,
                }
            }
            // now we have a match
            return true;
        }
        false
    }

    pub fn from_call(call: &[u8]) -> Self {
        let mut idx = [0u8; 6];
        let r = Self::normalize_callsign(call, &mut idx);
        debug_assert!(r);

        let mut val = idx[0] as u32;
        val = val * 36 + idx[1] as u32;
        val = val * 10 + idx[2] as u32;
        val = val * 27 + idx[3] as u32;
        val = val * 27 + idx[4] as u32;
        val = val * 27 + idx[5] as u32;
        Self(val + Self::VALUE_CALLSIGN_RANGE.start())
    }

    pub fn to_string<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        assert!(out.len() == 7);

        if self.0 == Self::VALUE_DE {
            out.copy_from_slice(b"DE     ");
        } else if self.0 == Self::VALUE_QRZ {
            out.copy_from_slice(b"QRZ    ");
        } else if self.0 == Self::VALUE_CQ {
            out.copy_from_slice(b"CQ     ");
        } else if Self::VALUE_CQNUM_RANGE.contains(&self.0) {
            out.copy_from_slice(b"CQ ___ ");
            let mut num = self.0 - Self::VALUE_CQNUM_RANGE.start();
            out[3] = b'0' + (num % 10) as u8;
            num /= 10;
            out[4] = b'0' + (num % 10) as u8;
            num /= 10;
            out[5] = b'0' + num as u8;
        } else if Self::VALUE_CQZONE_RANGE.contains(&self.0) {
            out.copy_from_slice(b"CQ ____");
            num_to_alphas(self.0 - Self::VALUE_CQZONE_RANGE.start(), &mut out[3..7]);
        } else if Self::VALUE_HASH_RANGE.contains(&self.0) {
            out.copy_from_slice(b"<.....>");
            // TODO:
        } else if Self::VALUE_CALLSIGN_RANGE.contains(&self.0) {
            Self::num_to_call(self.0 - Self::VALUE_CALLSIGN_RANGE.start(), &mut out[..6]);
            out[6] = b' ';
        } else {
            // panic!("invalid C28 value: {}", self.0);
            out.copy_from_slice(b"ERROR  ");
        }

        out
    }

    fn num_to_call(mut val: u32, out: &mut [u8]) -> &[u8] {
        assert!(out.len() >= 6);

        let mut idx = [0u8; 6];

        idx[5] = (val % 27) as u8;
        val /= 27;
        idx[4] = (val % 27) as u8;
        val /= 27;
        idx[3] = (val % 27) as u8;
        val /= 27;
        idx[2] = (val % 10) as u8;
        val /= 10;
        idx[1] = (val % 36) as u8;
        val /= 36;
        idx[0] = (val % 39) as u8;

        for (i, &x) in idx.iter().enumerate() {
            out[i] = Self::CHARS[i][x];
        }
        out
    }
}

fn alphas_to_num(seq: &[u8]) -> u32 {
    let mut val = 0;
    for &c in seq {
        val = val * 26 + (c - b'A') as u32;
    }
    val
}

fn num_to_alphas(mut val: u32, seq: &mut [u8]) {
    for x in seq.iter_mut().rev() {
        *x = Chars::AlphaSpc[(val % 27) as u8];
        val /= 27;
    }
    debug_assert_eq!(val, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c28_normalize() {
        const TESTCASES: &[(&[u8], &[u8])] = &[
            (b"JA1ZLO", b"JA1ZLO"),
            (b"8N1N", b"8N1N  "),
            (b"K1ABC", b" K1ABC"),
        ];
        let mut buf1 = [0u8; 6];
        let mut buf2 = [0u8; 6];

        for (call, out) in TESTCASES {
            assert!(C28::normalize_callsign(call, &mut buf1));
            for i in 0..6 {
                buf2[i] = C28::CHARS[i][buf1[i]];
            }
            assert_eq!(out, out);
        }
    }

    #[test]
    fn test_c28() {
        let mut out = [0u8; 7];
        const TESTCASES: &[(&[u8], u32)] = &[
            (b"JA1ZLO", 149982772),
            (b"JJ1FYD", 151740002),
            (b"8N1N", 74587795),
            (b"K1ABC", 10214965),
        ];

        for (call, num) in TESTCASES {
            let c = C28::from_call(call);
            assert_eq!(c.0, *num);

            let c = C28(*num);
            c.to_string(&mut out);
            let mut out: &[u8] = &out;
            while out.first() == Some(&b' ') {
                out = &out[1..];
            }
            while out.last() == Some(&b' ') {
                out = &out[..out.len() - 1];
            }
            assert_eq!(call, &out);
        }
    }

    #[test]
    fn test_c28_to_string() {
        let mut out = [0u8; 7];

        const TESTCASES: &[(u32, &[u8])] = &[
            (C28::VALUE_DE, b"DE     "),
            (C28::VALUE_QRZ, b"QRZ    "),
            (C28::VALUE_CQ, b"CQ     "),
            (*C28::VALUE_CQNUM_RANGE.start(), b"CQ 000 "),
            (1004, b"CQ    A"),
            (1031, b"CQ   AA"),
            (1760, b"CQ  AAA"),
            (21443, b"CQ AAAA"),
            (532443, b"CQ ZZZZ"),
            // (*C28::VALUE_HASH_RANGE.start(), b"<.....>"),
        ];
        for (num, ret) in TESTCASES {
            let c = C28(*num);
            c.to_string(&mut out);
            assert_eq!(out, *ret);
        }
    }
}
