// 1byte floating point number
// used for save LLR values

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
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

impl Into<f32> for F8 {
    fn into(self) -> f32 {
        self.0
    }
}
