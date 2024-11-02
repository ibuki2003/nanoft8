pub const CALLSIGN_STDLEN: usize = 6;
pub const CALLSIGN_MAXLEN: usize = 11;

pub type FullCallsign = [u8; CALLSIGN_MAXLEN];

mod c28;
pub use c28::C28;

mod c58;
pub use c58::C58;

pub mod hash;
pub mod hashtable;
