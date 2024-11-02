#![cfg_attr(feature = "no_std", no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod decoder;
pub mod protocol;

mod bits;
pub use bits::Bitset;

pub mod minifloat;

pub mod util;
