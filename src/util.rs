pub fn trim_u8str(mut str: &[u8]) -> &[u8] {
    while str.first().is_some_and(|&x| x == b' ') {
        str = &str[1..];
    }
    while str.last().is_some_and(|&x| x == b' ') {
        str = &str[..str.len() - 1];
    }
    str
}

#[inline]
pub fn write_slice(out: &mut [u8], s: &[u8]) -> Option<usize> {
    if out.len() < s.len() {
        return None;
    }
    out[..s.len()].copy_from_slice(s);
    Some(s.len())
}
