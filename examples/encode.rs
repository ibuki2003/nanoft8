use nanoft8::{
    protocol::{
        crc::{add_crc, check_crc},
        ldpc,
        message::{Message, C28, G15},
    },
    Bitset, F8,
};

fn main() {
    let msg = Message::StdMsg {
        call1: C28::from_call(b"JA1ZLO").unwrap(),
        call1_r: false,
        call2: C28::from_call(b"JA1YWX").unwrap(),
        call2_r: false,
        r: true,
        grid: G15::from_grid_string(b"PM95"),
    };

    let mut str = [0; 64];
    msg.to_string(&mut str);
    let str = String::from_utf8_lossy(&str);
    println!("msg: {}", str);

    let mut bs = msg.encode();

    add_crc(&mut bs);

    let mut buf = [false; 174];
    ldpc::encode(&bs, &mut buf);

    // println!("encoded: {}", bs);
    let str = buf
        .iter()
        .map(|&b| if b { '1' } else { '0' })
        .collect::<String>();
    println!("encoded: {}", str);

    let llr = buf
        .iter()
        .map(|&b| if b { 1.0 } else { -1.0 })
        .map(F8::from_f32)
        .collect::<Vec<_>>();
    println!("err: {}", ldpc::check(&buf));

    let mut bs = Bitset::default();
    ldpc::solve(&llr, &mut bs);

    println!("crc: {:?}", check_crc(&bs));

    let msg = Message::decode(&bs).unwrap();
    let mut str = [0; 64];
    msg.to_string(&mut str);
    let str = String::from_utf8_lossy(&str);
    println!("decoded: {}", str);
}
