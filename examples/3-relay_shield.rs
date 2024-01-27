//! This example shows how to communicate with STM32F302R8-Nucleo
//! board + Arduino 4-relay shield.
//! 1. Program Nucleo board with `nucleo.bin` from this directory.
//! 2. Connect Nucleo board to PC using USB cable.
//! 3. Change COM port name.
//! 3. Run this example `cargo run --example 3-relay_shield`.
//! https://www.seeedstudio.com/Relay-Shield-v3-0.html
//! https://www.st.com/en/evaluation-tools/nucleo-f302r8.html

extern crate rand;
extern crate serialport;
extern crate wake_rs;

use rand::Rng;
use std::thread;
use std::time::Duration;
use wake_rs::{Decode, Encode, Packet, DATA_MAX_LEN};

const MODE_MAX: u8 = 5;
const RELAY_NUM: u8 = 4;
const DO_NOT_CHECK_RX_SIZE: usize = 0x100;

struct RelayCmd {
    need_rx: usize,
    command: u8,
    data_tx: Option<Vec<u8>>,
}

impl RelayCmd {
    fn send(&self, p: &mut dyn serialport::SerialPort) -> Result<Option<Vec<u8>>, &str> {
        let wp = Packet {
            address: None,
            command: self.command,
            data: self.data_tx.clone(),
        };
        p.write(wp.encode().unwrap().as_mut_slice())
            .expect("failed to write");

        let mut rx = [0; DATA_MAX_LEN];
        let rx_len = p.read(&mut rx).expect("failed to read");
        let decoded = rx[..rx_len].to_vec().decode().expect("failed to decode");
        if decoded.command != wp.command {
            return Err("RX_CMD != TX_CMD");
        }
        if self.need_rx == DO_NOT_CHECK_RX_SIZE {
            return Ok(decoded.data);
        }
        match decoded.data {
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
        }
    }
}

fn get_info(p: &mut dyn serialport::SerialPort) -> Result<String, &'static str> {
    let get_info = RelayCmd {
        need_rx: DO_NOT_CHECK_RX_SIZE,
        command: 0x02,
        data_tx: None,
    };
    let rx = get_info.send(p).unwrap();
    Ok(String::from_utf8(rx.unwrap()).expect("Found invalid UTF-8"))
}

fn set_relay(p: &mut dyn serialport::SerialPort, relay: u8, mode: u8) -> Result<(), &'static str> {
    let data_tx = vec![relay, mode];
    let set_relay: RelayCmd = RelayCmd {
        need_rx: 1,
        command: 0x10,
        data_tx: Some(data_tx),
    };
    set_relay.send(p).unwrap();
    Ok(())
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut port = serialport::new("COM5", 115200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let info = get_info(&mut *port).expect("Relay shield is not connected");
    println!("Device info: {}", info);

    loop {
        let relay = rng.gen_range(0..RELAY_NUM);
        let mode = rng.gen_range(0..MODE_MAX);
        let delay = rng.gen_range(200..3000);

        set_relay(&mut *port, relay, mode).expect("Connection error");
        thread::sleep(Duration::from_millis(delay));
        println!("Relay {} Mode {} Delay {}", relay, mode, delay);
    }
}
