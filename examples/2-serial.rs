//! This example shows how to use wake_rs library along with serial port.
//! 1. Connect RX and TX pins.
//! 2. Run this example `cargo run --example 2-serial`

extern crate serialport;
extern crate wake_rs;

use std::io::Write;
use std::thread;
use std::time::Duration;
use wake_rs::{Decode, Encode, Packet};

fn print_packet(header: &str, v: Option<&Vec<u8>>) {
    print!("\n{}:\t", header);
    match v {
        Some(data) => {
            for x in data {
                print!("{:02X} ", x);
            }
        }
        None => print!("[]"),
    }
}

fn main() {
    let cmd_version = Packet {
        address: None,
        command: 0x01,
        data: None,
    }
    .encode()
    .unwrap();

    let cmd_start = Packet {
        address: None,
        command: 0x02,
        data: Some(vec![10, 10]),
    }
    .encode()
    .unwrap();

    let cmd_stop = Packet {
        address: None,
        command: 0x02,
        data: Some(vec![0, 0]),
    }
    .encode()
    .unwrap();

    let mut commands = [cmd_version, cmd_start, cmd_stop];

    let mut port = serialport::new("COM4", 9600)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open port");

    let mut rx: Vec<u8> = vec![0; 64];
    let mut state: usize = 0;
    loop {
        port.write(commands[state].as_mut_slice())
            .expect("failed to write message");
        let n = port.read(rx.as_mut_slice()).unwrap();
        print_packet("RAW RX", Some(&rx[..n].to_vec()));
        let d = &rx[..n].to_vec().decode().unwrap();
        print!("\nDecoded CMD {}", d.command);
        print_packet("Decoded data", d.data.as_ref());
        state = if state >= 2 { 0 } else { state + 1 };
        print!("\n------------");
        thread::sleep(Duration::from_millis(5000));
    }
}
