/// **signed** value that can be converted to/from f32
pub trait FloatS: From<f32> + Into<f32> + Default + Copy {}
impl FloatS for f32 {}

/// **unsigned** value that can be converted to/from f32
pub trait FloatU: From<f32> + Into<f32> + PartialOrd + Default + Copy {}
impl FloatU for f32 {}
