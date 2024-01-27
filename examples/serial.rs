//! This example shows how to use wake_rs library along with serial port.
extern crate serialport;
extern crate wake;

use serialport::prelude::*;
use std::io::Write;
use std::thread;
use std::time::Duration;
use wake::{Decode, Encode};

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
    let settings = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(10),
    };

    let port = serialport::open_with_settings("/dev/ttyS2", &settings);
    let cmd_version = wake::Packet {
        address: None,
        command: 0x01,
        data: None,
    }
    .encode();
    let cmd_start = wake::Packet {
        address: None,
        command: 0x02,
        data: Some(vec![10, 10]),
    }
    .encode();
    let cmd_stop = wake::Packet {
        address: None,
        command: 0x02,
        data: Some(vec![0, 0]),
    }
    .encode();

    match port {
        Ok(mut p) => {
            println!("Port is opened");
            let mut rx: Vec<u8> = vec![0; 64];
            let mut cmd: Vec<u8>;

            let mut state: u8 = 0;
            loop {
                match state {
                    1 => cmd = cmd_start.clone(),
                    2 => cmd = cmd_stop.clone(),
                    _ => {
                        state = 0;
                        cmd = cmd_version.clone()
                    }
                }
                state += 1;
                p.write(cmd.as_mut_slice())
                    .expect("failed to write message");
                if let Ok(t) = p.read(rx.as_mut_slice()) {
                    print_packet("RAW RX", Some(&rx[..t].to_vec()));
                    if let Ok(d) = &rx[..t].to_vec().decode() {
                        print!("\nDecoded CMD {}", d.command);
                        print_packet("Decoded data", d.data.as_ref());
                    }
                }
                print!("\n------------");
                thread::sleep(Duration::from_millis(5000));
            }
        }
        Err(_e) => panic!("Error: Port is not available"),
    }
}
