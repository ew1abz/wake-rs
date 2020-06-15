//! This example shows how to communicate with the device that contans 4 relays
//! (Nucleo board and arduino relay shield).
//! TODO: add links

extern crate clap;
extern crate rand;
extern crate serialport;
extern crate wake;

use clap::clap_app;
use rand::Rng;
use serialport::prelude::*;
// use std::error::Error;
use std::thread;
use std::time::Duration;
use wake::{Decode, Encode};

const MODE_MAX: u8 = 5;
const RELAY_NUM: u8 = 4;
const DO_NOT_CHECK_RX_SIZE: usize = 0x100;

struct WakeCmd {
    need_rx: usize,
    _timeout: u32,
    wp: wake::Packet,
}

const GET_INFO: WakeCmd = WakeCmd {
    need_rx: DO_NOT_CHECK_RX_SIZE,
    _timeout: 0,
    wp: wake::Packet {
        address: None,
        command: 0x02,
        data: None,
    },
};

const SET_RELAY: WakeCmd = WakeCmd {
    need_rx: 1,
    _timeout: 0,
    wp: wake::Packet {
        address: None,
        command: 0x10,
        data: None,
    },
};

impl WakeCmd {
    fn send<'a>(&self, p: &mut dyn serialport::SerialPort) -> Result<Option<Vec<u8>>, &str> {
        let mut encoded = self.wp.encode();
        p.write(encoded.as_mut_slice()).expect("failed to write"); // TODO: use ?

        let mut rx = [0; wake::DATA_MAX_LEN];
        let rx_len = p.read(&mut rx).expect("failed to read"); // TODO: use ?
                                                               // if rx_len.is_err() {
                                                               //     return Err("Failed to read");
                                                               // }
                                                               //let rxv = rx[..rx_len.unwrap()].to_vec();
        let decoded = rx[..rx_len].to_vec().decode().expect("failed to decode"); // TODO: use ?
                                                                                 //let decoded = rxv.clone().decode()?;
                                                                                 //let packet = decoded.unwrap();
        if decoded.command != self.wp.command {
            return Err("RX_CMD != TX_CMD");
        }
        if self.need_rx == DO_NOT_CHECK_RX_SIZE {
            return Ok(decoded.data);
        }
        return match decoded.data {
            Some(data) => {
                if data.len() != self.need_rx as usize {
                    Err("need_rx != real_rx")
                } else {
                    Ok(Some(data))
                }
            }
            None => {
                if self.need_rx as usize != 0 {
                    Err("need_rx != 0")
                } else {
                    Ok(None)
                }
            }
        };
    }
}

fn get_info(p: &mut dyn serialport::SerialPort) -> Result<String, &'static str> {
    let rx = GET_INFO.send(p)?;
    return Ok(String::from_utf8(rx.unwrap()).expect("Found invalid UTF-8"));
}

fn set_relay(p: &mut dyn serialport::SerialPort, relay: u8, mode: u8) -> Result<(), &'static str> {
    SET_RELAY.wp.data = Some(vec![relay, mode]);
    SET_RELAY.send(p)?;
    Ok(())
}

fn main() {
    let matches = clap_app!(myapp =>
        (@arg port_name: -p default_value("COM18") "Port name")
        (@arg baud_rate: -b default_value("115200") "Baud rate")
    )
    .get_matches();

    let name = matches.value_of("port_name").unwrap();
    let rate = matches
        .value_of("baud_rate")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let settings = SerialPortSettings {
        baud_rate: rate,
        timeout: Duration::from_millis(10),
        ..Default::default()
    };

    let mut rng = rand::thread_rng();
    let mut port = serialport::open_with_settings(name, &settings).expect("Port is not available");
    let info = get_info(&mut *port).expect("Relay shield is not connected");
    println!("Port: {} Baudrate: {} Device: {}", name, rate, info);

    loop {
        let relay = rng.gen_range(0, RELAY_NUM);
        let mode = rng.gen_range(0, MODE_MAX);
        let delay = rng.gen_range(200, 3000);

        set_relay(&mut *port, relay, mode).expect("Connection error");
        thread::sleep(Duration::from_millis(delay));
        println!("Relay {} Mode {} Delay {}", relay, mode, delay);
    }
}
