//! This example shows how to communicate with device that contans 4 relays.
extern crate serialport;
extern crate wake;

use serialport::prelude::*;
use std::thread;
use std::time::Duration;

const C_INFO: u8 = 0x02;
const C_RELAYS_SET: u8 = 0x10;
const MODE_MAX: u8 = 5;

struct WakeCmd {
    code: u8,
    need_rx: u8,
    tx: Vec<u8>,
}

const DO_NOT_CHECK_RX_SIZE: u8 = 0xFF;

fn send_cmd<'a>(p: &mut serialport::SerialPort, cmd: WakeCmd) -> Result<Option<Vec<u8>>, &str> {
    let wp = wake::Packet {
        address: None,
        command: cmd.code,
        data: Some(vec![0x00, 0xeb]),
    };
    let mut encoded = wake::encode_packet(wp);
    p.write(encoded.as_mut_slice())
        .expect("failed to write message");
    let mut rx = [0; 0xff];
    if let Ok(t) = p.read(&mut rx) {
        if let Ok(decoded) = wake::decode_packet(&rx[..t].to_vec()) {
            if decoded.command != cmd.code {
                return Err("RX_CMD != TX_CMD");
            }
            if cmd.need_rx != DO_NOT_CHECK_RX_SIZE {
                match decoded.data {
                    Some(data) => if data.len() != cmd.need_rx as usize {
                            return Err("need_rx != real_rx");
                            } else {return Ok(Some(data));}
                    None => if cmd.need_rx as usize != 0 {
                        return Err("need_rx != real_rx");
                    }
                }
            }            
        }
    }
    Err("Cannot read")
}

fn get_info(p: &mut SerialPort) -> Result<String, &str> {
    let cmd_get_info = WakeCmd {
        code: C_INFO,
        need_rx: DO_NOT_CHECK_RX_SIZE,
        tx: vec![],
    };
    match send_cmd(p, cmd_get_info) {
        Ok(rx) => {
            let s = String::from_utf8(rx.unwrap()).expect("Found invalid UTF-8");
            return Ok(s);
        }
        Err(e) => return Err(e),
    }
}

fn set_relay(p: &mut SerialPort, relay: u8, mode: u8) -> Result<(), &str> {
    let cmd_set_relay = WakeCmd {
        code: C_RELAYS_SET,
        need_rx: 1,
        tx: vec![relay, mode],
    };
    match send_cmd(p, cmd_set_relay) {
        Ok(_) => return Ok(()),
        Err(e) => return Err(e),
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

    let port = serialport::open_with_settings("&ports[0].port_name", &settings);
    match port {
        Ok(mut p) => {
            println!("Port is opened");
            match get_info(&mut *p) {
                Ok(s) => println!("Shield is connected: {:?}", s),
                Err(e) => panic!("Error [get_info]: {:?}", e),
            }

            let mut relay = 0;
            let mut mode = 0;
            loop {
                match set_relay(&mut *p, relay, mode) {
                    Ok(_) => print!("\nRelay {} Mode {}", relay, mode),
                    Err(e) => panic!("Error [set_relay]: {:?}", e),
                }
                mode += 1;
                if mode == MODE_MAX {
                    mode = 0;
                    relay = (relay + 1) & 3;
                }
                thread::sleep(Duration::from_millis(3000));
            }
        }
        Err(_e) => panic!("Error: Port is not available"),
    }
}
