extern crate serialport;
extern crate wake;

use std::io::Write;
use serialport::prelude::*;
use std::time::Duration;
use std::thread;
use wake::*;

fn print_packet(header: &str, v: &Vec<u8>) {
    print!("\n{}:\t", header);
    for x in v {
        print!("{:02X} ", x);
    }
}

/// Main function doc string
fn main() {
    let settings = SerialPortSettings {
        baud_rate: BaudRate::Baud115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(10),
    };

    if let Ok(ports) = serialport::available_ports() {
        match ports.len() {
            0 => panic!("No ports found."),
            1 => println!("Found 1 port:"),
            n => println!("Found {} ports:", n),
        };
        for p in ports.iter() {
            println!("{:?}", p);
        }
        let mut port = serialport::open_with_settings(&ports[0].port_name, &settings);
        match port {
            Ok(mut p) => {
                println!("Port is opened");
                let mut tx: Vec<u8> = vec![0; 2];
                let mut rx: Vec<u8> = vec![0; 64];

                loop {
                    tx[1] += 1; // relay mode
                    tx[1] &= 7;
                    if tx[1] == 0 {
                        tx[0] += 1; // relay number
                        tx[0] &= 3;
                    }
                    print!("\nRelay {} Mode {}", tx[0], tx[1]);
                    let mut encoded = encode_packet(0x10, &tx);
                    p.write(encoded.as_mut_slice()).expect("failed to write message");
                    if let Ok(t) = p.read(rx.as_mut_slice()) {
                        print_packet("RAW RX", &rx[..t].to_vec());
                        if let Ok(d) = decode_packet(&rx[..t].to_vec()) {
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
}
