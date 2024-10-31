use core::ops::Shr as _;

use crate::protocol::message::chars::Chars;

// NOTE: returns ~0 if the callsign is invalid
pub fn hash_callsign(mut str: &[u8]) -> u32 {
    // trim whitespace
    while str.first().is_some_and(|&x| x == b' ') {
        str = &str[1..];
    }
    while str.last().is_some_and(|&x| x == b' ') {
        str = &str[..str.len() - 1];
    }

    if str.len() > 11 {
        return !0;
    }

    let mut n = 0u64;

    for c in str.iter() {
        match Chars::AlnumSs.find(*c) {
            Some(v) => {
                n = n.wrapping_mul(38).wrapping_add(v as u64);
            }
            None => return !0,
        };
    }
    for _ in str.len()..11 {
        n = n.wrapping_mul(38);
    }
    n.wrapping_mul(47055833459).shr(64 - 22) as u32 & ((1 << 22) - 1)
}
