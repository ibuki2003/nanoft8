use std::io::BufRead as _;

use nanoft8::{
    protocol::{message::Message, BODY_BITS},
    Bitset,
};

fn main() {
    let mut bs = Bitset::default();
    // read lines from stdin; infinite loop
    let stdin = std::io::stdin();
    let lines = stdin.lock().lines();

    for line in lines {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.len() < BODY_BITS {
            eprintln!("line too short: {}", line);
            continue;
        }
        for (i, c) in line[..BODY_BITS].chars().enumerate() {
            bs.set(i, c == '1');
        }
        let msg = if let Some(v) = Message::decode(&bs) {
            v
        } else {
            eprintln!("failed to decode");
            continue;
        };

        println!("{}", msg);
    }
}
