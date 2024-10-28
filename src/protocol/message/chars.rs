use core::ops::Index;

const CHARS_ALNUM_SPC: &[u8] = b" 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CHARS_ALNUM: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CHARS_NUMERIC: &[u8] = b"0123456789";
const CHARS_ALPHA_SPC: &[u8] = b" ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CHARS_ALNUM_SS: &[u8] = b" 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ/";
const CHARS_FULL: &[u8] = b" 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?";

#[derive(Clone, Copy)]
pub enum Chars {
    AlnumSpc,
    Alnum,
    Numeric,
    AlphaSpc,
    AlnumSs,
    Full,
}

impl Chars {
    #[inline]
    pub const fn size(&self) -> usize {
        self.get_str().len()
    }

    #[inline]
    pub fn find(&self, c: u8) -> Option<u8> {
        match self {
            Self::AlnumSpc => idx_alnum_spc(c),
            Self::Alnum => idx_alnum(c),
            Self::Numeric => idx_numeric(c),
            Self::AlphaSpc => idx_alpha_spc(c),
            Self::AlnumSs => idx_alnum_ss(c),
            Self::Full => idx_full(c),
        }
    }

    #[inline]
    pub const fn get(&self, idx: u8) -> u8 {
        self.get_str()[idx as usize]
    }

    #[inline]
    pub const fn get_str(&self) -> &'static [u8] {
        match self {
            Self::AlnumSpc => CHARS_ALNUM_SPC,
            Self::Alnum => CHARS_ALNUM,
            Self::Numeric => CHARS_NUMERIC,
            Self::AlphaSpc => CHARS_ALPHA_SPC,
            Self::AlnumSs => CHARS_ALNUM_SS,
            Self::Full => CHARS_FULL,
        }
    }
}

impl Index<u8> for Chars {
    type Output = u8;
    fn index(&self, idx: u8) -> &u8 {
        &self.get_str()[idx as usize]
    }
}

fn idx_alnum(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'Z' => Some(c - b'A' + 10),
        b'a'..=b'z' => Some(c - b'a' + 10), // case insensitive
        _ => None,
    }
}

fn idx_alnum_spc(c: u8) -> Option<u8> {
    if c == b' ' {
        Some(0)
    } else {
        idx_alnum(c).map(|x| x + 1)
    }
}

fn idx_numeric(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        _ => None,
    }
}

fn idx_alpha_spc(c: u8) -> Option<u8> {
    match c {
        b' ' => Some(0),
        b'A'..=b'Z' => Some(c - b'A' + 1),
        b'a'..=b'z' => Some(c - b'a' + 1), // case insensitive
        _ => None,
    }
}

fn idx_alnum_ss(c: u8) -> Option<u8> {
    if c == b'/' {
        Some(CHARS_ALNUM_SS.len() as u8 - 1)
    } else {
        idx_alnum_spc(c)
    }
}

fn idx_full(c: u8) -> Option<u8> {
    match c {
        b'+' => Some(CHARS_ALNUM_SPC.len() as u8),
        b'-' => Some(CHARS_ALNUM_SPC.len() as u8 + 1),
        b'.' => Some(CHARS_ALNUM_SPC.len() as u8 + 2),
        b'/' => Some(CHARS_ALNUM_SPC.len() as u8 + 3),
        b'?' => Some(CHARS_ALNUM_SPC.len() as u8 + 4),
        _ => idx_alnum_spc(c),
    }
}
