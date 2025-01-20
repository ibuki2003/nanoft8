#![feature(generic_const_exprs)]
use nanoft8::{
    protocol::{
        message::{callsign::hashtable::HashTable, Message},
        BODY_BITS,
    },
    Bitset,
};
use std::io::BufRead as _;

fn main() {
    let mut bs = Bitset::default();
    // read lines from stdin; infinite loop
    let stdin = std::io::stdin();
    let lines = stdin.lock().lines();

    // you can use either HashTable or BTreeMap
    let mut hashes = HashTable::<[u8; 11], 2, 4>::new();
    // let mut hashes = BTreeMap::<u32, [u8; 11]>::new();

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

        let mut str = [0; 64];
        let n = msg.write_str(&mut str, Some(&hashes)).unwrap();
        let str = std::str::from_utf8(&str[..n]).unwrap();

        println!("{}", str);

        msg.register_callsigns(&mut hashes);
    }
}
