// 1byte floating point number
// used for save LLR values

#[derive(Copy, Clone, PartialEq, Default)]
pub struct _F8<const SIGNED: bool, const EXP_SIZE: usize, const EXP_BIAS: u8>(pub u8);

impl<const SIGNED: bool, const EXP_SIZE: usize, const EXP_BIAS: u8>
    _F8<SIGNED, EXP_SIZE, EXP_BIAS>
{
    const FRAC_SIZE: usize = 8 - EXP_SIZE - (SIGNED as usize);

    pub const ZERO: Self = Self(0);
}

impl<const SIGNED: bool, const EXP_SIZE: usize, const EXP_BIAS: u8>
    _F8<SIGNED, EXP_SIZE, EXP_BIAS>
{
    pub fn as_f32(self) -> f32 {
        if self.0 & 0x7f == 0 {
            return if (self.0 >> 7) == 0 { 0. } else { -0. };
        }

        if self.0 & 0x7f == 0x7f {
            return if (self.0 >> 7) == 0 {
                f32::INFINITY
            } else {
                f32::NEG_INFINITY
            };
        }

        let sign = if SIGNED { (self.0 >> 7) as u32 } else { 0 };

        let mut exp = ((self.0 >> Self::FRAC_SIZE) & ((1 << EXP_SIZE) - 1)) + (127 - EXP_BIAS);
        let mut frac = self.0 & ((1 << Self::FRAC_SIZE) - 1);

        if exp == 127 - EXP_BIAS {
            // denormalized
            let s = frac.leading_zeros() as u8 - (8 - Self::FRAC_SIZE as u8) + 1;
            exp -= s - 1;
            frac = (frac << s) & ((1 << Self::FRAC_SIZE) - 1);
        }

        let frac = (frac as u32) << (23 - Self::FRAC_SIZE);
        let exp = exp as u32;
        let bits = sign << 31 | exp << 23 | frac;
        f32::from_bits(bits)
    }

    pub fn from_f32(f: f32) -> Self {
        let bits = f.to_bits();

        let sign = if SIGNED { (bits >> 31) as u8 } else { 0 };
        let exp = ((bits >> 23) & ((1 << 8) - 1)) as u8;
        let frac = (bits >> (23 - Self::FRAC_SIZE)) as u8 & ((1 << Self::FRAC_SIZE) - 1);

        if exp >= (1 << EXP_SIZE) + 127 - EXP_BIAS {
            // overflow
            return if SIGNED {
                Self(sign << 7 | 0x7f)
            } else {
                Self(0xff)
            };
        }
        if exp <= 127 - EXP_BIAS {
            // denormalization
            let shift = 127 - EXP_BIAS - exp + 1;
            let frac = if shift <= Self::FRAC_SIZE as u8 {
                (frac | (1 << Self::FRAC_SIZE)) >> shift
            } else {
                0
            };
            return Self(sign << 7 | frac);
        }

        // here subtraction won't overflow
        let exp = exp - (127 - EXP_BIAS);

        Self(sign << 7 | exp << Self::FRAC_SIZE | frac)
    }

    pub fn to_bits(self) -> u8 {
        self.0
    }

    pub fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
}

impl<const SIGNED: bool, const EXP_SIZE: usize, const EXP_BIAS: u8> From<f32>
    for _F8<SIGNED, EXP_SIZE, EXP_BIAS>
{
    fn from(f: f32) -> Self {
        Self::from_f32(f)
    }
}
impl<const SIGNED: bool, const EXP_SIZE: usize, const EXP_BIAS: u8>
    From<_F8<SIGNED, EXP_SIZE, EXP_BIAS>> for f32
{
    fn from(f: _F8<SIGNED, EXP_SIZE, EXP_BIAS>) -> Self {
        f.as_f32()
    }
}

pub type F8 = _F8<true, 5, 24>;
pub type Fu8 = _F8<false, 6, 24>;

// floating point number that can be converted to f32
pub trait FloatS: From<f32> + Into<f32> + Default + Copy {}
impl FloatS for f32 {}
impl FloatS for F8 {}

pub trait FloatU: From<f32> + Into<f32> + Default + Copy {
    // ordering is necessary in decoder
    type Inner: Ord;
    fn to_inner(&self) -> Self::Inner;
}
impl FloatU for f32 {
    type Inner = u32;
    fn to_inner(&self) -> Self::Inner {
        self.to_bits()
    }
}
impl FloatU for Fu8 {
    type Inner = u8;
    fn to_inner(&self) -> Self::Inner {
        self.0
    }
}
