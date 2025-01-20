use crate::{protocol::message::chars::Chars, util::trim_u8str};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C58(pub u64);

// standard callsigns
impl C58 {
    pub fn from_call(call: &[u8]) -> Option<Self> {
        let call = trim_u8str(call);

        if call.len() > 11 {
            return None;
        }

        let mut val = 0;

        for &c in call {
            val = val * 38 + Chars::AlnumSs.find(c)? as u64;
        }
        Some(Self(val))
    }

    pub fn write_str(&self, out: &mut [u8]) -> Option<usize> {
        let mut v = self.0;
        let mut len = 0;
        for c in out.iter_mut() {
            if v == 0 {
                break;
            }
            *c = Chars::AlnumSs.get((v % 38) as u8);
            v /= 38;
            len += 1;
        }
        if v != 0 {
            return None;
        }
        out[..len].reverse();
        Some(len)
    }
}

#[cfg(not(feature = "no_std"))]
impl core::fmt::Display for C58 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut str = [0u8; 11];
        self.write_str(&mut str);
        f.write_str(core::str::from_utf8(&str).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c58() {
        let testcases = &[(b"", 0)];

        let mut buf = [0; 11];
        for &(call, val) in testcases.iter() {
            let c58 = C58::from_call(call).unwrap();
            assert_eq!(c58.0, val);
            let n = c58.write_str(&mut buf).unwrap();
            assert_eq!(&buf[..n], call);
        }
    }
}
