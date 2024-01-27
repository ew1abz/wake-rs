extern crate wake;

use wake::{Decode, Encode};

fn print_hex_buffer(header: &str, v: &Vec<u8>) {
    print!("\n{}\t", header);
    for x in v {
        print!("{:02X} ", x);
    }
    println!("");
}

/// Simple wake_rs API demo
fn main() {
    let wp = wake::Packet {
        address: Some(0x12),
        command: 3,
        data: Some(vec![0x00, 0xeb]),
    };

    let encoded = wp.encode();
    print_hex_buffer("Encoded packet:\t", &encoded);

    let decoded = encoded.decode();
    println!("Decoded packet: {}", decoded.unwrap());
}
