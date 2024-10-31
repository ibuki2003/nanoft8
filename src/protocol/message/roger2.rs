// RRR message
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

    pub fn to_string(&self, out: &mut [u8]) {
        assert_eq!(out.len(), 4);
        match self {
            Self::BLANK => out.copy_from_slice(b"    "),
            Self::RRR => out.copy_from_slice(b"RRR "),
            Self::RR73 => out.copy_from_slice(b"RR73"),
            Self::V73 => out.copy_from_slice(b"73  "),
        }
    }
}
