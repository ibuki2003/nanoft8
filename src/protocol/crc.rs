use super::{BODY_BITS, CRC_BITS};
use crate::Bitset;

// pub const POLY: u16 = 0x6757;
pub const POLY: u16 = 0x6757;

// message and CRC are given as a bitset
pub fn check_crc(data: &Bitset) -> bool {
    calc_crc(data) == data.slice(BODY_BITS, CRC_BITS) as u16
}

pub fn calc_crc(data: &Bitset) -> u16 {
    let mut crc = 0u32;
    // TODO: speed up
    for i in 0..95 {
        crc ^= if i < BODY_BITS && data.get(i) { 1 } else { 0 };
        if crc & (1 << 13) != 0 {
            crc = (crc << 1) ^ POLY as u32;
        } else {
            crc <<= 1;
        }
    }

    (crc & ((1 << CRC_BITS) - 1)) as u16
}

pub fn add_crc(data: &mut Bitset) {
    let crc = calc_crc(data);
    data.set_slice(BODY_BITS, CRC_BITS, crc as u32);
}
