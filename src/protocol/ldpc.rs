use super::{BODY_BITS, CRC_BITS, PAYLOAD_BITS};
use crate::{Bitset, F8};

const V_SIZE: usize = PAYLOAD_BITS;
const C_SIZE: usize = 83;
const MSG_BITS: usize = BODY_BITS + CRC_BITS;

const MAX_ITER: usize = 100;
const MAX_ITER_NO_PROGRESS: usize = 10;

type FullMessageBits = [bool; V_SIZE];
fn check(message: &FullMessageBits) -> u8 {
    let mut count = 0;
    for row in TABLE_CV.iter() {
        let sum = row.iter().fold(false, |acc, j| {
            if (*j as usize) < V_SIZE {
                acc ^ message[*j as usize]
            } else {
                acc
            }
        });
        if sum {
            count += 1;
        }
    }
    count
}

// NOTE: original solve algorithm from kgoba/ft8_lib Copyright (c) 2018 Kārlis Goba
// solve the parity check equations
// out: the message bits
// returns the number of errors
pub fn solve(message: &[F8], out: &mut Bitset) -> u8 {
    debug_assert!(message.len() == V_SIZE);

    let mut message_f32 = [0.0f32; V_SIZE];
    for (i, &b) in message.iter().enumerate() {
        message_f32[i] = b.into();
    }

    let mut plain = [false; V_SIZE];

    let mut tov = [[0.0f32; TABLE_VC_LEN]; V_SIZE];
    let mut toc = [[0.0f32; TABLE_CV_LEN]; C_SIZE];

    let mut min_err = C_SIZE as u8;
    let mut last_err = min_err;
    let mut count_no_progress = 0;

    for _ in 0..MAX_ITER {
        // check

        for i in 0..V_SIZE {
            plain[i] = (message_f32[i] + tov[i].iter().sum::<f32>()) > 0.0;
        }

        last_err = check(&plain);

        if last_err == 0 {
            break;
        }
        if last_err < min_err {
            min_err = last_err;
            count_no_progress = 0;
        } else {
            count_no_progress += 1;
            if count_no_progress >= MAX_ITER_NO_PROGRESS {
                break;
            }
        }

        // improvement iteration

        for (m, row) in TABLE_CV.iter().enumerate() {
            for (i, &n) in row.iter().enumerate() {
                if n == EOL {
                    break;
                }
                let mut sum = message_f32[n as usize];
                for (j, &m1) in TABLE_VC[n as usize].iter().enumerate() {
                    if m1 != m as u8 {
                        sum += tov[n as usize][j];
                    }
                }
                toc[m][i] = (sum / -2.).tanh();
            }
        }

        for (n, row) in TABLE_VC.iter().enumerate() {
            for (i, &m) in row.iter().enumerate() {
                let mut sum = 1.0;
                for (j, &n1) in TABLE_CV[m as usize].iter().enumerate() {
                    if n1 == EOL {
                        break;
                    }
                    if n1 != n as u8 {
                        sum *= toc[m as usize][j];
                    }
                }
                tov[n][i] = -2. * sum.atanh();
            }
        }
    }
    for (i, &b) in plain[..MSG_BITS].iter().enumerate() {
        out.set(i, b);
    }
    last_err
}

const EOL: u8 = 0xff; // marker for the end of the table row

const TABLE_CV_LEN: usize = 7;
pub const TABLE_CV: [[u8; TABLE_CV_LEN]; C_SIZE] = [
    [3, 30, 58, 90, 91, 95, 152],
    [4, 31, 59, 92, 114, 145, EOL],
    [5, 23, 60, 93, 121, 150, EOL],
    [6, 32, 61, 94, 95, 142, EOL],
    [7, 24, 62, 82, 92, 95, 147],
    [5, 31, 63, 96, 125, 137, EOL],
    [4, 33, 64, 77, 97, 106, 153],
    [8, 34, 65, 98, 138, 145, EOL],
    [9, 35, 66, 99, 106, 125, EOL],
    [10, 36, 66, 86, 100, 138, 157],
    [11, 37, 67, 101, 104, 154, EOL],
    [12, 38, 68, 102, 148, 161, EOL],
    [7, 39, 69, 81, 103, 113, 144],
    [13, 40, 70, 87, 101, 122, 155],
    [14, 41, 58, 105, 122, 158, EOL],
    [0, 32, 71, 105, 106, 156, EOL],
    [15, 42, 72, 107, 140, 159, EOL],
    [16, 36, 73, 80, 108, 130, 153],
    [10, 43, 74, 109, 120, 165, EOL],
    [44, 54, 63, 110, 129, 160, 172],
    [7, 45, 70, 111, 118, 165, EOL],
    [17, 35, 75, 88, 112, 113, 142],
    [18, 37, 76, 103, 115, 162, EOL],
    [19, 46, 69, 91, 137, 164, EOL],
    [1, 47, 73, 112, 127, 159, EOL],
    [20, 44, 77, 82, 116, 120, 150],
    [21, 46, 57, 117, 126, 163, EOL],
    [15, 38, 61, 111, 133, 157, EOL],
    [22, 42, 78, 119, 130, 144, EOL],
    [18, 34, 58, 72, 109, 124, 160],
    [19, 35, 62, 93, 135, 160, EOL],
    [13, 30, 78, 97, 131, 163, EOL],
    [2, 43, 79, 123, 126, 168, EOL],
    [18, 45, 80, 116, 134, 166, EOL],
    [6, 48, 57, 89, 99, 104, 167],
    [11, 49, 60, 117, 118, 143, EOL],
    [12, 50, 63, 113, 117, 156, EOL],
    [23, 51, 75, 128, 147, 148, EOL],
    [24, 52, 68, 89, 100, 129, 155],
    [19, 45, 64, 79, 119, 139, 169],
    [20, 53, 76, 99, 139, 170, EOL],
    [34, 81, 132, 141, 170, 173, EOL],
    [13, 29, 82, 112, 124, 169, EOL],
    [3, 28, 67, 119, 133, 172, EOL],
    [0, 3, 51, 56, 85, 135, 151],
    [25, 50, 55, 90, 121, 136, 167],
    [51, 83, 109, 114, 144, 167, EOL],
    [6, 49, 80, 98, 131, 172, EOL],
    [22, 54, 66, 94, 171, 173, EOL],
    [25, 40, 76, 108, 140, 147, EOL],
    [1, 26, 40, 60, 61, 114, 132],
    [26, 39, 55, 123, 124, 125, EOL],
    [17, 48, 54, 123, 140, 166, EOL],
    [5, 32, 84, 107, 115, 155, EOL],
    [27, 47, 69, 84, 104, 128, 157],
    [8, 53, 62, 130, 146, 154, EOL],
    [21, 52, 67, 108, 120, 173, EOL],
    [2, 12, 47, 77, 94, 122, EOL],
    [30, 68, 132, 149, 154, 168, EOL],
    [11, 42, 65, 88, 96, 134, 158],
    [4, 38, 74, 101, 135, 166, EOL],
    [1, 53, 85, 100, 134, 163, EOL],
    [14, 55, 86, 107, 118, 170, EOL],
    [9, 43, 81, 90, 110, 143, 148],
    [22, 33, 70, 93, 126, 152, EOL],
    [10, 48, 87, 91, 141, 156, EOL],
    [28, 33, 86, 96, 146, 161, EOL],
    [29, 49, 59, 85, 136, 141, 161],
    [9, 52, 65, 83, 111, 127, 164],
    [21, 56, 84, 92, 139, 158, EOL],
    [27, 31, 71, 102, 131, 165, EOL],
    [27, 28, 83, 87, 116, 142, 149],
    [0, 25, 44, 79, 127, 146, EOL],
    [16, 26, 88, 102, 115, 152, EOL],
    [50, 56, 97, 162, 164, 171, EOL],
    [20, 36, 72, 137, 151, 168, EOL],
    [15, 46, 75, 129, 136, 153, EOL],
    [2, 23, 29, 71, 103, 138, EOL],
    [8, 39, 89, 105, 133, 150, EOL],
    [14, 57, 59, 73, 110, 149, 162],
    [17, 41, 78, 143, 145, 151, EOL],
    [24, 37, 64, 98, 121, 159, EOL],
    [16, 41, 74, 128, 169, 171, EOL],
];

// Each row corresponds to a codeword bit.
// The numbers indicate which three LDPC parity checks (rows in Nm) refer to the codeword bit.
// 1-origin.
pub const TABLE_VC_LEN: usize = 3;
pub const TABLE_VC: [[u8; 3]; V_SIZE] = [
    [15, 44, 72],
    [24, 50, 61],
    [32, 57, 77],
    [0, 43, 44],
    [1, 6, 60],
    [2, 5, 53],
    [3, 34, 47],
    [4, 12, 20],
    [7, 55, 78],
    [8, 63, 68],
    [9, 18, 65],
    [10, 35, 59],
    [11, 36, 57],
    [13, 31, 42],
    [14, 62, 79],
    [16, 27, 76],
    [17, 73, 82],
    [21, 52, 80],
    [22, 29, 33],
    [23, 30, 39],
    [25, 40, 75],
    [26, 56, 69],
    [28, 48, 64],
    [2, 37, 77],
    [4, 38, 81],
    [45, 49, 72],
    [50, 51, 73],
    [54, 70, 71],
    [43, 66, 71],
    [42, 67, 77],
    [0, 31, 58],
    [1, 5, 70],
    [3, 15, 53],
    [6, 64, 66],
    [7, 29, 41],
    [8, 21, 30],
    [9, 17, 75],
    [10, 22, 81],
    [11, 27, 60],
    [12, 51, 78],
    [13, 49, 50],
    [14, 80, 82],
    [16, 28, 59],
    [18, 32, 63],
    [19, 25, 72],
    [20, 33, 39],
    [23, 26, 76],
    [24, 54, 57],
    [34, 52, 65],
    [35, 47, 67],
    [36, 45, 74],
    [37, 44, 46],
    [38, 56, 68],
    [40, 55, 61],
    [19, 48, 52],
    [45, 51, 62],
    [44, 69, 74],
    [26, 34, 79],
    [0, 14, 29],
    [1, 67, 79],
    [2, 35, 50],
    [3, 27, 50],
    [4, 30, 55],
    [5, 19, 36],
    [6, 39, 81],
    [7, 59, 68],
    [8, 9, 48],
    [10, 43, 56],
    [11, 38, 58],
    [12, 23, 54],
    [13, 20, 64],
    [15, 70, 77],
    [16, 29, 75],
    [17, 24, 79],
    [18, 60, 82],
    [21, 37, 76],
    [22, 40, 49],
    [6, 25, 57],
    [28, 31, 80],
    [32, 39, 72],
    [17, 33, 47],
    [12, 41, 63],
    [4, 25, 42],
    [46, 68, 71],
    [53, 54, 69],
    [44, 61, 67],
    [9, 62, 66],
    [13, 65, 71],
    [21, 59, 73],
    [34, 38, 78],
    [0, 45, 63],
    [0, 23, 65],
    [1, 4, 69],
    [2, 30, 64],
    [3, 48, 57],
    [0, 3, 4],
    [5, 59, 66],
    [6, 31, 74],
    [7, 47, 81],
    [8, 34, 40],
    [9, 38, 61],
    [10, 13, 60],
    [11, 70, 73],
    [12, 22, 77],
    [10, 34, 54],
    [14, 15, 78],
    [6, 8, 15],
    [16, 53, 62],
    [17, 49, 56],
    [18, 29, 46],
    [19, 63, 79],
    [20, 27, 68],
    [21, 24, 42],
    [12, 21, 36],
    [1, 46, 50],
    [22, 53, 73],
    [25, 33, 71],
    [26, 35, 36],
    [20, 35, 62],
    [28, 39, 43],
    [18, 25, 56],
    [2, 45, 81],
    [13, 14, 57],
    [32, 51, 52],
    [29, 42, 51],
    [5, 8, 51],
    [26, 32, 64],
    [24, 68, 72],
    [37, 54, 82],
    [19, 38, 76],
    [17, 28, 55],
    [31, 47, 70],
    [41, 50, 58],
    [27, 43, 78],
    [33, 59, 61],
    [30, 44, 60],
    [45, 67, 76],
    [5, 23, 75],
    [7, 9, 77],
    [39, 40, 69],
    [16, 49, 52],
    [41, 65, 67],
    [3, 21, 71],
    [35, 63, 80],
    [12, 28, 46],
    [1, 7, 80],
    [55, 66, 72],
    [4, 37, 49],
    [11, 37, 63],
    [58, 71, 79],
    [2, 25, 78],
    [44, 75, 80],
    [0, 64, 73],
    [6, 17, 76],
    [10, 55, 58],
    [13, 38, 53],
    [15, 36, 65],
    [9, 27, 54],
    [14, 59, 69],
    [16, 24, 81],
    [19, 29, 30],
    [11, 66, 67],
    [22, 74, 79],
    [26, 31, 61],
    [23, 68, 74],
    [18, 20, 70],
    [33, 52, 60],
    [34, 45, 46],
    [32, 58, 75],
    [39, 42, 82],
    [40, 41, 62],
    [48, 74, 82],
    [19, 43, 47],
    [41, 48, 56],
];

#[expect(clippy::unusual_byte_groupings)]
const TABLE_GEN: [[u32; (C_SIZE + 31) / 32]; MSG_BITS] = [
    [0b10100000101001010000100011011000, 0b11000111001000000010100101111110, 0b1000011101010111000_0000000000000],
    [0b01100001110011110001000100010100, 0b00010111100110010100110010011011, 0b0001100110110101101_0000000000000],
    [0b01000011111011100010000001100101, 0b01101011001000101110111111100010, 0b0110101111010110111_0000000000000],
    [0b01110000011111000100110000000101, 0b11001111001101010001011110111100, 0b0001011100111100100_0000000000000],
    [0b00111010000101001011111101101001, 0b11001110011111001100110110111101, 0b1001001010101101100_0000000000000],
    [0b01100100010011111101000000011110, 0b01111110100000110000111110010011, 0b0001110111101110010_0000000000000],
    [0b11010100110011100001000011101011, 0b10111100000100011100111110110101, 0b1100100001001000110_0000000000000],
    [0b10011110010001001111111001100101, 0b00001000000011101110100100101011, 0b1011010010011011100_0000000000000],
    [0b00001010001101101110101101001010, 0b00011110011110001011011110011110, 0b1101111010101101101_0000000000000],
    [0b00001101010010001000101011110010, 0b10110010001101010101000011010100, 0b0000001000000000010_0000000000000],
    [0b10111110001101111110000011011111, 0b01111001001110000000100111101000, 0b0010100111101110100_0000000000000],
    [0b01011111011001001101011010111111, 0b00011010101100100100111100010111, 0b0011000101111110100_0000000000000],
    [0b11011100011001001010011100001110, 0b11000000011100101010010000010000, 0b1010101000011101001_0000000000000],
    [0b01111111110011111100100110010011, 0b00000111011111111100110001001101, 0b0000001001010010011_0000000000000],
    [0b00110010100010101001001100001011, 0b10111111011110111011001101101010, 0b1111101111011000100_0000000000000],
    [0b10011000100010101000011101000011, 0b01100110010010110100001010111000, 0b1110110100011001100_0000000000000],
    [0b10001101110101000110010101010010, 0b01101010011101000011101111010001, 0b0010000001100001111_0000000000000],
    [0b10110101000101001001100100000101, 0b11001110011001101011001101110101, 0b1001110000101111011_0000000000000],
    [0b01001011000000111001110010010000, 0b01110000001100100001101000100100, 0b0001100011000100010_0000000000000],
    [0b00100001111010011010111001001000, 0b11010001101001010010000000110011, 0b1100100111111001000_0000000000000],
    [0b10100111110111100111000000111000, 0b10101110000000011100010101010111, 0b0101110111100101111_0000000000000],
    [0b11001100010010010011010001100100, 0b01101011000100010001111100000010, 0b0100000101000000000_0000000000000],
    [0b11000011000011111001111111111010, 0b10101000010100110110000111110101, 0b0011001010001110110_0000000000000],
    [0b00110000001101001001010111000011, 0b00010000111110011001001001011000, 0b0011100110001111110_0000000000000],
    [0b00001111100011011000001011000010, 0b00011010001101110011000100010101, 0b0110100011000101010_0000000000000],
    [0b01011111100000000000011011110101, 0b01110011100010000100011100101110, 0b1001110100111101001_0000000000000],
    [0b00011011100100000101100110000111, 0b11110010001110101000001011011110, 0b1001010111111010110_0000000000000],
    [0b10011011000000101101111100011001, 0b00000110111000000000111010100010, 0b1100100001000010001_0000000000000],
    [0b01011010010000010010100110100001, 0b10110100110110100010100001101100, 0b1111011101110000110_0000000000000],
    [0b01001011100000101011001111000100, 0b01111101011101110011010101000110, 0b0111101111001101111_0000000000000],
    [0b01101010001110000001001010111100, 0b00010111001010011001001001111011, 0b1010000000100101101_0000000000000],
    [0b10000101000110110011111100111110, 0b01111101010001111010000101010110, 0b0011000100000101111_0000000000000],
    [0b10101001011001110001100101111000, 0b10111001010111101100001111111010, 0b1111101000001001010_0000000000000],
    [0b00111001010001010101100110111011, 0b10110001010110101010001011001101, 0b1111001101011001011_0000000000000],
    [0b11101011110000010111001100000100, 0b11011111011001001101010101110000, 0b0110111011001010010_0000000000000],
    [0b10110111100100011011111100011101, 0b01100010111001001001111000110100, 0b1101101001001000001_0000000000000],
    [0b10110110011101110011010000001101, 0b10101111100101000011000101110100, 0b0000010001011110011_0000000000000],
    [0b11000010001100111010000101010010, 0b10110010010111100110100011111100, 0b1001010110100000000_0000000000000],
    [0b10100101000111110101000000011100, 0b10101000000011011110110100011111, 0b0011100101101101010_0000000000000],
    [0b11100101100100100110011000011010, 0b00101001000000000000101111000100, 0b0101010001101011111_0000000000000],
    [0b01010110000111111010101011100011, 0b01101000010100011001101111010001, 0b0010100010101011100_0000000000000],
    [0b01011001000100001111111001100011, 0b00011011001011001100111100111100, 0b1000011000110111011_0000000000000],
    [0b10100010001010000110011110101100, 0b00001110011100010110101101010111, 0b1010011011010010000_0000000000000],
    [0b10000001000000101001111110001000, 0b11010000111110001000011000001100, 0b0101110111110101000_0000000000000],
    [0b00010101111000000110101101100001, 0b01101111110001001101111001000001, 0b0011101010111010001_0000000000000],
    [0b00110001110100010100010110100101, 0b11010100011101001111101001110010, 0b1111110100101000110_0000000000000],
    [0b01100000111101100000011111011110, 0b10110000011111100110110011101000, 0b1111100100100011101_0000000000000],
    [0b10111001001001111001100101001110, 0b10111011111110101110001111101010, 0b1001001010101001001_0000000000000],
    [0b10001001101000011101001100001000, 0b11001010010010000011011111101011, 0b1100111001000111111_0000000000000],
    [0b11100100101011000001111011100110, 0b10100010100111011110001011101100, 0b1010101011100110101_0000000000000],
    [0b10110110111010111000101111000111, 0b00001010111111110011001100001101, 0b0000100000101001011_0000000000000],
    [0b01101111001010010010110111111110, 0b01101110010111011011010100000101, 0b1110000010000001111_0000000000000],
    [0b11110000101100000111101100110101, 0b10100011111101100011010011110101, 0b0001010010011101011_0000000000000],
    [0b00111011110000100010111111000001, 0b01100101110011100110011010111110, 0b1110100101111000100_0000000000000],
    [0b10000111010110000101011000100000, 0b11010010100110110110100000001110, 0b0000111100010000001_0000000000000],
    [0b01011100101001111111100110000001, 0b01010100111101101100000001000110, 0b0000001111110110011_0000000000000],
    [0b10011111011101111101000110001001, 0b10101011111101001010001111001000, 0b0010100101011101001_0000000000000],
    [0b10111111010101010011000111110001, 0b01111000011010001100001111111001, 0b1111100111110111110_0000000000000],
    [0b11101110100001101011110111111110, 0b01100101011101001001000001000111, 0b0100111111000000101_0000000000000],
    [0b11011011011111001100100011011011, 0b11000110101110010110110001111010, 0b0001010001001110001_0000000000000],
    [0b00001100010111000100101100101100, 0b11010111001111110001001010111101, 0b0000011101111010000_0000000000000],
    [0b10101110111110010011100111111110, 0b11011000100110110100011111101001, 0b1100100000001011111_0000000000000],
    [0b01010001101111101011001110010001, 0b01101000011010111011100100101000, 0b0110101001100010110_0000000000000],
    [0b11011101111100011100011011101011, 0b00001110101000000100111101110011, 0b0111000111011011001_0000000000000],
    [0b00000011110001100100111100000110, 0b01100011110111101110011000011001, 0b0111110001110000000_0000000000000],
    [0b01000110000101001010000011001100, 0b00000101100011101100001010101011, 0b0000011101011001111_0000000000000],
    [0b00010011011001100000010101101011, 0b10101111010000100011101101010010, 0b0100001011011110010_0000000000000],
    [0b01110111000101110011010011010001, 0b10110110000101001101101010010000, 0b1000111110101011011_0000000000000],
    [0b10010110110101101101010000100001, 0b01111010101110000101100100101111, 0b1110010001010101011_0000000000000],
    [0b01010110010100100010101110011111, 0b00001001110111101111100111111010, 0b1110110100010001011_0000000000000],
    [0b00011010010001111001011010111001, 0b10101010001001100100010111000101, 0b0010001111001110100_0000000000000],
    [0b10001000001011111101010001101010, 0b01100110100110101001110010011010, 0b1101110110001001111_0000000000000],
    [0b11110001000010000011101110100011, 0b00000110110111010111100010101100, 0b1011000011110000100_0000000000000],
    [0b10011001110101110101111111110011, 0b11000001110011000010011011111011, 0b1010010111010101001_0000000000000],
    [0b10100100001010110000100110010100, 0b01100111010000011010111001010111, 0b1011011001000011101_0000000000000],
    [0b11000110011110001001001000101000, 0b00010001100100111110101010000111, 0b1000001100010101010_0000000000000],
    [0b00000111100100001110011110111000, 0b10100000010111000000011000001010, 0b0000111010111110111_0000000000000],
    [0b00011100011101101001011001000000, 0b00011001101001101110010111100110, 0b1111000011011111110_0000000000000],
    [0b11011010110101110001011110100001, 0b00001000111000001010110100000001, 0b1010001111000010110_0000000000000],
    [0b01111100000011110110011010110011, 0b01100010101110011100001100001111, 0b1010100010111101011_0000000000000],
    [0b00111011101101010011100101101001, 0b00001101011100001110101110110100, 0b0101111010110000100_0000000000000],
    [0b10010101100000000100001110101001, 0b10111011010001001111101100100100, 0b1001100101001000011_0000000000000],
    [0b10110000110111011111001101010011, 0b00001100010101011001101110000111, 0b1101111101100010001_0000000000000],
    [0b11110010010001100111110110110101, 0b00001000010001001001000111010111, 0b1110011111001011100_0000000000000],
    [0b10100110100010000001011111101000, 0b01101001011101011010101100110110, 0b0010010011001000000_0000000000000],
    [0b10110010001000101000110010111011, 0b00000001001011000001000000100111, 0b0100100111000010100_0000000000000],
    [0b11011001001111101100011110011111, 0b11001101010010011100100101011000, 0b0100010001101101110_0000000000000],
    [0b11101011111110111001001111100000, 0b00011010110111010001100100111001, 0b0001101001001101000_0000000000000],
    [0b10101111011011010110011011001110, 0b10111101000000010001000110000000, 0b1100010011011010010_0000000000000],
    [0b10100001001001000111000110001100, 0b11001101010010000110001101011100, 0b1000101101000110010_0000000000000],
    [0b01011111000101000110000011000101, 0b00001110100010100100001101101011, 0b1001110101011000100_0000000000000],
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_table_vc_cv() {
        for (i, &row) in TABLE_VC.iter().enumerate() {
            for &c in row.iter() {
                assert!(TABLE_CV[c as usize].contains(&(i as u8)));
            }
        }
    }

    #[test]
    fn test_table_rev() {
        for (i, &row) in TABLE_GEN.iter().enumerate() {
            for &hrow in TABLE_CV.iter() {
                let mut bit = false;
                for &x in hrow.iter() {
                    if x == EOL { continue; }
                    if x < MSG_BITS as u8 {
                        if x == i as u8 {
                            bit = !bit;
                        }
                    } else {
                        let xx = x - MSG_BITS as u8;
                        let xi = (xx / 32) as usize;
                        let xj = 31 - xx % 32;
                        let b = row[xi] & (1 << xj) != 0;
                        if b {
                            bit = !bit;
                        }
                    }
                }
                assert!(!bit);
            }
        }
    }
}
