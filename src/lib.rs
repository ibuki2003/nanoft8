#![cfg_attr(feature = "no_std", no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs, inherent_associated_types)]

pub mod decoder;
pub mod protocol;

mod bits;
pub use bits::Bitset;

pub mod float;

pub mod util;
