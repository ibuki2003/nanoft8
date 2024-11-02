
pub fn trim_u8str(mut str: &[u8]) -> &[u8] {
    while str.first().is_some_and(|&x| x == b' ') {
        str = &str[1..];
    }
    while str.last().is_some_and(|&x| x == b' ') {
        str = &str[..str.len() - 1];
    }
    str
}
