extern crate serialport;
extern crate wake_rs;

use serialport::prelude::*;
use std::io::Write;
use std::thread;
use std::time::Duration;
use wake_rs::*;

fn print_packet(header: &str, v: &Vec<u8>) {
    print!("\n{}:\t", header);
    for x in v {
        print!("{:02X} ", x);
    }
}

/// Main function doc string
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
    let cmd_version = wake::encode_packet(wake::Packet {
        addr: None,
        command: 0x01,
        data: None,
    });
    let cmd_start = wake::encode_packet(wake::Packet {
        addr: None,
        command: 0x02,
        data: Some(vec![10, 10]),
    });
    let cmd_stop = wake::encode_packet(wake::Packet {
        addr: None,
        command: 0x02,
        data: Some(vec![0, 0]),
    });

    match port {
        Ok(mut p) => {
            println!("Port is opened");
            let mut rx: Vec<u8> = vec![0; 64];
            let mut cmd: Vec<u8>;

            let mut state: u8 = 0;
            loop {
                // let mut encoded = encode_packet(0x01, &tx);
                // let mut encoded = wake::encode_packet(cmdVersion);
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
                    print_packet("RAW RX", &rx[..t].to_vec());
                    if let Ok(d) = wake::decode_packet(&rx[..t].to_vec()) {
                        print!("\nDecoded CMD {}", d.0);
                        print_packet("Decoded data", &d.1);
                    }
                }
                print!("\n------------");
                thread::sleep(Duration::from_millis(5000));
            }
        }
        Err(_e) => panic!("Error: Port not available"),
    }
}
