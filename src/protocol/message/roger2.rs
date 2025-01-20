use crate::util::write_slice;

// RRR message
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum R2 {
    BLANK,
    RRR,
    RR73,
    V73,
}

impl R2 {
    pub fn to_val(&self) -> u8 {
        match self {
            Self::BLANK => 0,
            Self::RRR => 1,
            Self::RR73 => 2,
            Self::V73 => 3,
        }
    }

    pub fn from_val(v: u8) -> Self {
        match v {
            0 => Self::BLANK,
            1 => Self::RRR,
            2 => Self::RR73,
            3 => Self::V73,
            _ => unreachable!(),
        }
    }

    pub fn write_str(&self, out: &mut [u8]) -> Option<usize> {
        match self {
            Self::BLANK => write_slice(out, b" "),
            Self::RRR => write_slice(out, b"RRR"),
            Self::RR73 => write_slice(out, b"RR73"),
            Self::V73 => write_slice(out, b"73"),
        }
    }
}

#[cfg(not(feature = "no_std"))]
impl core::fmt::Display for R2 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut s = [0; 4];
        let n = self.write_str(&mut s).unwrap();
        f.write_str(core::str::from_utf8(&s[..n]).unwrap())
    }
}
