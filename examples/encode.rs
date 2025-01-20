#![feature(generic_const_exprs)]
use nanoft8::protocol::{
    crc::{add_crc, check_crc},
    ldpc,
    message::{callsign::C28, Message, G15},
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
    msg.write_str(&mut str, None::<&()>);
    let str = String::from_utf8_lossy(&str);
    println!("msg: {}", str);

    let bs = msg.encode();
    println!("{:p}", bs.0.as_ptr());

    let bs = add_crc(bs);
    println!("{:p}", bs.0.as_ptr());

    let buf = ldpc::encode(&bs);

    // println!("encoded: {}", bs);
    println!("encoded: {}", buf);

    let llr = (0..174)
        .map(|i| buf.get(i))
        .map(|b| if b { 1.0 } else { -1.0 })
        .collect::<Vec<_>>();
    println!("err: {}", ldpc::check(&buf));

    let (bs, _err) = ldpc::solve(&llr);

    println!("crc: {:?}", check_crc(&bs));

    let msg = Message::decode(&bs.with_size()).unwrap();
    let mut str = [0; 64];
    msg.write_str(&mut str, None::<&()>);
    let str = String::from_utf8_lossy(&str);
    println!("decoded: {}", str);
}
