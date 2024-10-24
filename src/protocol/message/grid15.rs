// grid locator 4
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct G15(pub u16);

impl G15 {
    const ALPHA_CNT: u16 = 18; // A..R
    const GRID_MAX: u16 = Self::ALPHA_CNT * Self::ALPHA_CNT * 10 * 10;

    const VALUE_RRR: u16 = Self::GRID_MAX + 1;
    const VALUE_RR73: u16 = Self::GRID_MAX + 2;
    const VALUE_V73: u16 = Self::GRID_MAX + 3;

    pub const RRR: Self = Self(Self::GRID_MAX + 1);
    pub const RR73: Self = Self(Self::GRID_MAX + 2);
    pub const V73: Self = Self(Self::GRID_MAX + 3);

    pub fn to_string(&self, out: &mut [u8]) {
        debug_assert!(out.len() == 4);

        out.fill(b' ');

        match self.0 {
            0..Self::GRID_MAX => {
                let mut val = self.0;
                out[3] = b'0' + (val % 10) as u8;
                val /= 10;
                out[2] = b'0' + (val % 10) as u8;
                val /= 10;
                out[1] = b'A' + (val % Self::ALPHA_CNT) as u8;
                val /= Self::ALPHA_CNT;
                out[0] = b'A' + (val % Self::ALPHA_CNT) as u8;
            }
            Self::GRID_MAX => out.copy_from_slice(b"    "),
            Self::VALUE_RRR => out.copy_from_slice(b"RRR "),
            Self::VALUE_RR73 => out.copy_from_slice(b"RR73"),
            Self::VALUE_V73 => out.copy_from_slice(b"73  "),
            _ => {
                let mut report = self.0 as i16 - Self::GRID_MAX as i16 - 35;
                out[0] = if report < 0 { b'-' } else { b'+' };
                report = report.abs();
                out[1] = b'0' + (report / 10) as u8;
                out[2] = b'0' + (report % 10) as u8;
            }
        };
    }

    pub fn from_grid_string(str: &[u8]) -> Self {
        debug_assert!(str.len() == 4);
        let mut val = 0;
        val += (str[0] - b'A') as u16;
        val *= Self::ALPHA_CNT;
        val += (str[1] - b'A') as u16;
        val *= 10;
        val += (str[2] - b'0') as u16;
        val *= 10;
        val += (str[3] - b'0') as u16;
        Self(val)
    }

    pub fn from_report(report: i16) -> Self {
        debug_assert!((-30..=99).contains(&report));
        let val = (report + 35).unsigned_abs();
        Self(Self::GRID_MAX + val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid15() {
        let mut buf = [0; 4];

        let testcases = &[
            (b"JO22", G15::from_grid_string(b"JO22")),
            (b"-30 ", G15::from_report(-30)),
            (b"+00 ", G15::from_report(0)),
            (b"+99 ", G15::from_report(99)),
            (b"RRR ", G15::RRR),
            (b"RR73", G15::RR73),
            (b"73  ", G15::V73),
        ];

        for (str, g) in testcases {
            g.to_string(&mut buf);
            assert_eq!(String::from_utf8_lossy(&buf), String::from_utf8_lossy(*str));
        }

    }
}
