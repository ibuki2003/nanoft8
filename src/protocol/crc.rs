use super::{MessageBits, MessageBitsWithCRC, BODY_BITS, CRC_BITS};

// pub const POLY: u16 = 0x6757;
pub const POLY: u16 = 0x6757;

// message and CRC are given as a bitset
pub fn check_crc(data: &MessageBitsWithCRC) -> bool {
    calc_crc(&data.with_size()) == data.slice(BODY_BITS, CRC_BITS) as u16
}

pub fn calc_crc(data: &MessageBits) -> u16 {
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

pub fn add_crc(data: MessageBits) -> MessageBitsWithCRC {
    // let mut ret = data.with_size();
    let mut ret = unsafe { core::mem::transmute::<MessageBits, MessageBitsWithCRC>(data) };
    let crc = calc_crc(&data);
    ret.set_slice(BODY_BITS, CRC_BITS, crc as u32);
    ret
}
