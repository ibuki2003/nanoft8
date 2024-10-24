// 1byte floating point number
// used for save LLR values

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct F8(pub f32); // temporarily use f32

impl F8 {
    pub const ZERO: Self = Self(0.0);

    pub fn as_f32(self) -> f32 {
        self.0
    }

    pub fn from_f32(f: f32) -> Self {
        Self(f)
    }
}

impl From<f32> for F8 {
    fn from(f: f32) -> Self {
        Self(f)
    }
}

impl From<F8> for f32 {
    fn from(val: F8) -> Self {
        val.0
    }
}

#[cfg(not(feature = "no_std"))]
impl core::fmt::Debug for F8 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}
