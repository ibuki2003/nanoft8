use crate::{protocol::message::chars::Chars, util::trim_u8str};

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

    pub fn to_string<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        assert!(out.len() == 11);
        let mut v = self.0;
        for c in out.iter_mut().rev() {
            *c = Chars::AlnumSs.get((v % 38) as u8);
            v /= 38;
        }
        out
    }
}
