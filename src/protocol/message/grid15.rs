// grid locator 4
pub struct G15(pub u16);

impl G15 {
    const ALPHA_CNT: u16 = 18; // A..R
    pub fn to_string(&self, out: &mut[u8]) {
        debug_assert!(out.len() == 4);
        let mut val = self.0;
        out[3] = b'0' + (val % 10) as u8;
        val /= 10;
        out[2] = b'0' + (val % 10) as u8;
        val /= 10;
        out[1] = b'A' + (val % Self::ALPHA_CNT) as u8;
        val /= Self::ALPHA_CNT;
        out[0] = b'A' + (val % Self::ALPHA_CNT) as u8;
    }

    pub fn from_string(str: &[u8]) -> Self {
        debug_assert!(str.len() == 4);
        let mut val = 0;
        val += (str[3] - b'0') as u16;
        val *= 10;
        val += (str[2] - b'0') as u16;
        val *= 10;
        val += (str[1] - b'A') as u16;
        val *= Self::ALPHA_CNT;
        val += (str[0] - b'A') as u16;
        Self(val)
    }
}
