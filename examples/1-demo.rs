//! This example shows how to encode/decode data.
//! 1. Run this example `cargo run --example 1-demo`

extern crate wake_rs;

use wake_rs::{Decode, Encode};

fn print_hex_buffer(header: &str, v: &Vec<u8>) {
    print!("\n{}", header);
    for x in v {
        print!("{:02X} ", x);
    }
    println!("");
}

/// Simple wake_rs API demo
fn main() {
    let wp = wake_rs::Packet {
        address: Some(0x12),
        command: 3,
        data: Some(vec![0x00, 0xeb]),
    };

    let encoded = wp.encode().unwrap();
    print_hex_buffer("Encoded packet:\t", &encoded);

    let decoded = encoded.decode();
    println!("Decoded packet:\n{}", decoded.unwrap());
}
