use super::FullCallsign;
use crate::{protocol::message::chars::Chars, util::trim_u8str};
use core::ops::Shr as _;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CallsignHash {
    H22(u32),
    H12(u16),
    H10(u16),
}

impl CallsignHash {
    pub const fn depth(&self) -> usize {
        match self {
            CallsignHash::H22(_) => 22,
            CallsignHash::H12(_) => 12,
            CallsignHash::H10(_) => 10,
        }
    }

    // NOTE: unknown bits are set to 0
    pub const fn as_h22(&self) -> u32 {
        match self {
            CallsignHash::H22(x) => *x,
            CallsignHash::H12(x) => (*x as u32) << 10,
            CallsignHash::H10(x) => (*x as u32) << 12,
        }
    }

    // NOTE: unknown bits are set to 0
    pub const fn as_h12(&self) -> u16 {
        match self {
            CallsignHash::H22(x) => (*x >> 10) as u16,
            CallsignHash::H12(x) => *x,
            CallsignHash::H10(x) => *x >> 2,
        }
    }

    // NOTE: unknown bits are set to 0
    pub const fn as_h10(&self) -> u16 {
        match self {
            CallsignHash::H22(x) => (*x >> 12) as u16,
            CallsignHash::H12(x) => *x >> 2,
            CallsignHash::H10(x) => *x,
        }
    }

    // returns range of possible 22-bit hashes
    pub const fn range(&self) -> core::ops::Range<u32> {
        match self {
            CallsignHash::H22(x) => *x..*x + 1,
            CallsignHash::H12(x) => (*x as u32) << 10..((*x as u32 + 1) << 10),
            CallsignHash::H10(x) => (*x as u32) << 12..((*x as u32 + 1) << 12),
        }
    }

    // returns whether the two hashes *may* represent the same callsign
    pub fn matches(&self, other: &CallsignHash) -> bool {
        let d = self.depth().min(other.depth());
        let v = self.as_h22().shr(22 - d);
        let w = other.as_h22().shr(22 - d);
        v == w
    }
}

// NOTE: returns ~0 if the callsign is invalid
pub fn hash_callsign(str: &[u8]) -> Option<CallsignHash> {
    let str = trim_u8str(str);

    if str.len() > 11 {
        return None;
    }

    let mut n = 0u64;

    for c in str.iter() {
        let v = Chars::AlnumSs.find(*c)?;
        n = n.wrapping_mul(38).wrapping_add(v as u64);
    }
    for _ in str.len()..11 {
        n = n.wrapping_mul(38);
    }
    Some(CallsignHash::H22(
        n.wrapping_mul(47055833459).shr(64 - 22) as u32 & ((1 << 22) - 1),
    ))
}

pub trait CallsignHashTable {
    // return first match
    fn find_hash(&self, hash: CallsignHash) -> Option<&FullCallsign>;
    fn add(&mut self, callsign: &FullCallsign) -> bool;
}

impl<const N: usize, const M: usize> CallsignHashTable
    for super::hashtable::HashTable<FullCallsign, N, M>
where
    [FullCallsign; super::hashtable::table_size(N)]: Sized,
{
    fn find_hash(&self, hash: CallsignHash) -> Option<&FullCallsign> {
        self.get_partial(hash.as_h22())
            .filter(|(&key, _)| hash.matches(&CallsignHash::H22(key)))
            .map(|(_, value)| value)
            .next()
    }

    fn add(&mut self, callsign: &FullCallsign) -> bool {
        let hash = hash_callsign(callsign);
        if hash.is_none() {
            return false;
        }
        let hash = hash.unwrap().as_h22();
        self.set(hash, *callsign);
        true
    }
}

#[cfg(not(feature = "no_std"))]
impl CallsignHashTable for std::collections::BTreeMap<u32, FullCallsign> {
    fn find_hash(&self, hash: CallsignHash) -> Option<&FullCallsign> {
        self.range(hash.range()).next().map(|(_, v)| v)
    }
    fn add(&mut self, callsign: &FullCallsign) -> bool {
        let hash = hash_callsign(callsign);
        if hash.is_none() {
            return false;
        }
        let hash = hash.unwrap().as_h22();
        self.insert(hash, *callsign);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_match() {
        assert!(CallsignHash::H22(0x000000).matches(&CallsignHash::H22(0x000000)));
        assert!(!CallsignHash::H22(0x000001).matches(&CallsignHash::H22(0x000000)));

        assert!(CallsignHash::H22(0x000001).matches(&CallsignHash::H12(0x000)));
        assert!(CallsignHash::H22(0x000400).matches(&CallsignHash::H12(0x001)));
        assert!(CallsignHash::H22(0x0007ff).matches(&CallsignHash::H12(0x001)));
        assert!(!CallsignHash::H22(0x000800).matches(&CallsignHash::H12(0x001)));

        assert!(CallsignHash::H12(0x000).matches(&CallsignHash::H12(0x000)));
        assert!(!CallsignHash::H12(0x001).matches(&CallsignHash::H12(0x000)));

        assert!(CallsignHash::H12(0x004).matches(&CallsignHash::H10(0x001)));
        assert!(CallsignHash::H12(0x007).matches(&CallsignHash::H10(0x001)));
        assert!(!CallsignHash::H12(0x008).matches(&CallsignHash::H10(0x001)));

        assert!(CallsignHash::H22(0x001000).matches(&CallsignHash::H10(0x001)));
        assert!(CallsignHash::H22(0x001fff).matches(&CallsignHash::H10(0x001)));
        assert!(!CallsignHash::H22(0x002000).matches(&CallsignHash::H10(0x001)));
    }

    #[test]
    fn test_hash() {
        assert_eq!(hash_callsign(b"JA1ZLO"), Some(CallsignHash::H22(3380585)));
        assert_eq!(hash_callsign(b"JA1ZLO/1"), Some(CallsignHash::H22(12904)));
        assert_eq!(hash_callsign(b"JJ1FYD"), Some(CallsignHash::H22(2882573)));
    }
}
