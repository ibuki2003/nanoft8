/**
* Packet structure:
* [Marker] [Payload1] [Marker] [Payload2] [Marker]
*
* Payload structure:
* [Body] [CRC] [FEC]
*/
pub const PAYLOAD_LEN: usize = 58;
pub const PAYLOAD_HALF_LEN: usize = PAYLOAD_LEN / 2;
pub const PAYLOAD_BITS: usize = PAYLOAD_LEN * FSK_DEPTH; // 8-ary
pub const BODY_BITS: usize = 77;
pub const CRC_BITS: usize = 14;

pub const FSK_DEPTH: usize = 3;
pub const FSK_ARITY: usize = 1 << FSK_DEPTH;

pub const COSTAS_SIZE: usize = 7;
pub const MARKER_COSTAS: [usize; COSTAS_SIZE] = [3, 1, 4, 0, 6, 5, 2];

pub const GRAY_CODE: [u8; FSK_ARITY] = [0b000, 0b001, 0b011, 0b010, 0b110, 0b100, 0b101, 0b111];

pub mod message;

pub mod crc;
pub mod ldpc;
