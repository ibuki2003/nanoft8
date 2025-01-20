use crate::Bitset;

/**
* Packet structure:
* [Marker] [Payload1] [Marker] [Payload2] [Marker]
*
* Payload structure:
* [Body] [CRC] [FEC]
*/
pub const MESSAGE_LEN: usize = PAYLOAD_LEN + COSTAS_SIZE * 3;
pub const PAYLOAD_LEN: usize = 58;
pub const PAYLOAD_HALF_LEN: usize = PAYLOAD_LEN / 2;
pub const PAYLOAD_BITS: usize = PAYLOAD_LEN * FSK_DEPTH; // 8-ary
pub const BODY_BITS: usize = 77;
pub const CRC_BITS: usize = 14;

pub const FSK_DEPTH: usize = 3;
pub const FSK_ARITY: usize = 1 << FSK_DEPTH;

pub const COSTAS_SIZE: usize = 7;
pub const MARKER_COSTAS: [usize; COSTAS_SIZE] = [3, 1, 4, 0, 6, 5, 2];

pub const GRAY_CODE: [u8; FSK_ARITY] = [0, 1, 3, 2, 5, 6, 4, 7];
pub const GRAY_CODE_INV: [u8; FSK_ARITY] = [0b000, 0b001, 0b011, 0b010, 0b110, 0b100, 0b101, 0b111];

pub type MessageBits = Bitset<BODY_BITS>;
pub type MessageBitsWithCRC = Bitset<{ BODY_BITS + CRC_BITS }>;
pub type FullMessageBits = Bitset<PAYLOAD_BITS>;

pub mod message;

pub mod crc;
pub mod ldpc;

pub fn encode_symbols(data: &FullMessageBits) -> [u8; MESSAGE_LEN] {
    let mut ret = [0; MESSAGE_LEN];

    // three markers
    for (i, v) in MARKER_COSTAS.iter().enumerate() {
        let v = *v as u8;
        ret[i] = v;
        ret[i + PAYLOAD_HALF_LEN + 7] = v;
        ret[i + PAYLOAD_LEN + 14] = v;
    }

    for (i, x) in ret[7..PAYLOAD_HALF_LEN + 7].iter_mut().enumerate() {
        let v = data.slice(i * FSK_DEPTH, FSK_DEPTH);
        *x = GRAY_CODE[v as usize];
    }
    for (i, x) in ret[PAYLOAD_HALF_LEN + 14..PAYLOAD_LEN + 14]
        .iter_mut()
        .enumerate()
    {
        let j = i + PAYLOAD_HALF_LEN;
        let v = data.slice(j * FSK_DEPTH, FSK_DEPTH);
        *x = GRAY_CODE[v as usize];
    }
    ret
}
