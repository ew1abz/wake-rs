#![crate_name = "wake"]
//! Wake protocol library

use std::fmt;

const FEND: u8 = 0xC0;
const FESC: u8 = 0xDB;
const TFEND: u8 = 0xDC;
const TFESC: u8 = 0xDD;
const ADDR_MASK: u8 = 0x80;
const CRC_INIT: u8 = 0xDE;
const PACKET_MIN_LEN: usize = 4;

const TOO_SHORT_PACKET: &'static str = "Too short packet";
const CANNOT_FIND_START: &'static str = "Cannot find start of a packet";
const DESTUFFING_FAILED: &'static str = "De-stuffing failed";
const WRONG_LEN: &'static str = "Wrong packet length";
const WRONG_CRC: &'static str = "Wrong CRC";

#[derive(Default)]
pub struct Packet {
    pub address: Option<u8>,
    pub command: u8,
    pub data: Option<Vec<u8>>,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ADDR: {} CMD: {} {}",
            match self.address {
                Some(a) => a,
                None => 0,
            },
            self.command,
            match &self.data {
                Some(d) => {
                    let a = "DATA: ";
                    for x in d {
                        format!("{} {:02X}", a, x);
                    }
                    a
                }
                None => "",
            }
        )
    }
}

/// Update CRC sum
///
/// # Arguments
///
/// * `crc` - pre-initialized crc
/// * `data` - new data
///
fn crc8(crc: &mut u8, data: u8) {
    let mut b = data;
    for _ in 0..8 {
        *crc = if (b ^ *crc) & 1 == 1 {
            ((*crc ^ 0x18) >> 1) | 0x80
        } else {
            (*crc >> 1) & !0x80
        };
        b = b >> 1;
    }
}

#[test]
fn crc8_test() {
    let mut crc: u8 = CRC_INIT;
    crc8(&mut crc, 0x00);
    assert!(crc == 0x48);
    crc8(&mut crc, 0x01);
    assert!(crc == 0xda);
    crc8(&mut crc, 0xff);
    assert!(crc == 0x1c);
    crc8(&mut crc, 0x55);
    assert!(crc == 0xda);
}

/// Calculate CRC sum of data in a vector
///
/// # Arguments
///
/// * `data: &Vec<u8>` - input data
///
/// # Output
///
/// * `u8` - Calculated CRC
///
fn crc(data: &Vec<u8>) -> u8 {
    let mut crc: u8 = CRC_INIT;
    for n in data {
        crc8(&mut crc, *n);
    }
    crc
}

#[test]
fn crc_test() {
    let xs = vec![1, 2, 3, 4, 5];
    assert!(crc(&xs) == 0xd6);
    let xs = vec![0xc0, 0x03, 0x00];
    assert!(crc(&xs) == 0xeb);
}

/// Byte stuffing in a vector
///
/// # Arguments
///
/// * `data: &Vec<u8>` - input data
///
/// # Output
///
/// * `Vec<u8>` - output data
///
fn stuffing(data: &Vec<u8>) -> Vec<u8> {
    let mut stuffed = vec![data[0]];
    for x in &data[1..] {
        match *x {
            FESC => {
                stuffed.push(FESC);
                stuffed.push(TFESC);
            }
            FEND => {
                stuffed.push(FESC);
                stuffed.push(TFEND);
            }
            _ => stuffed.push(*x),
        }
    }
    stuffed
}

#[test]
fn stuffing_test() {
    let a = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND]; // initial_data
    let b = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed_data
    assert_eq!(stuffing(&a), b);
}

/// Byte destuffing in a vector
///
/// # Arguments
///
/// * `data` - Input data
///
/// # Output
///
/// * `Option<Vec<u8>>` - Destuffed data wrapped in Option
///
fn destuffing(data: &Vec<u8>) -> Option<Vec<u8>> {
    let mut output: Vec<u8> = vec![];
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            FESC => {
                if i > (data.len() - 2) {
                    return None;
                }
                match data[i + 1] {
                    TFESC => {
                        output.push(FESC);
                        i += 1;
                    }
                    TFEND => {
                        output.push(FEND);
                        i += 1;
                    }
                    _ => return None,
                }
            }
            _ => output.push(data[i]),
        }
        i += 1;
    }
    Some(output)
}

#[test]
fn destuffing_test() {
    let t0 = vec![]; // empty
    let t1 = vec![0x34]; // 1 byte
    let t2 = vec![1, 2, 3, 4, 5, FEND]; // stuffed data without first FEND
    let t3 = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC]; // stuffed data without last byte
    let t4 = vec![FEND, FESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed data with missed 3rd byte
    let t5 = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC, TFEND]; // well stuffed data
    let a5 = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND]; // destuffed t5
    assert_eq!(destuffing(&t0), Some(vec![]));
    assert_eq!(destuffing(&t1), Some(t1));
    assert_eq!(destuffing(&t2), Some(t2));
    assert_eq!(destuffing(&t3), None);
    assert_eq!(destuffing(&t4), None);
    assert_eq!(destuffing(&t5), Some(a5));
}

/// Encode packet to wake format
///
/// # Arguments
///
/// * `packet` - Packet with address, command code and data. Address and data are not mandatory.
///
/// # Output
///
/// * `Vec<u8>` - Encoded data in wake format
///
/// # Example
///
/// ```
/// extern crate wake;
///
/// let p = wake::Packet{address: Some(0x12), command: 3, data: Some(vec!{0x00, 0xeb})};
/// let mut encoded_packet: Vec<u8> = wake::encode_packet(p);
/// ```
///
pub fn encode_packet(packet: Packet) -> Vec<u8> {
    let mut encoded_packet = vec![];
    // 1. FEND
    encoded_packet.push(FEND);
    // 2. Address, if exists
    match packet.address {
        Some(addr) => encoded_packet.push(addr as u8 | ADDR_MASK),
        None => {}
    }
    // 3. Command
    encoded_packet.push(packet.command);
    // 4. Data length; data, if exists
    match packet.data {
        Some(d) => {
            encoded_packet.push(d.len() as u8);
            encoded_packet.extend(d.iter().cloned());
        }
        None => {
            encoded_packet.push(0);
        }
    }
    // 5. CRC
    let crc = crc(&encoded_packet);
    encoded_packet.push(crc);
    // 6. Stuffing
    stuffing(&encoded_packet)
}

#[test]
fn encode_packet_test_wp() {
    let wp = Packet {
        address: Some(0x12),
        command: 3,
        data: Some(vec![0x00, 0xeb]),
    };
    assert_eq!(
        encode_packet(wp),
        vec![FEND, 0x92, 0x03, 0x02, 0x00, 0xeb, 114]
    );
}

/// Decode packet from wake format to wake structure
///
/// # Arguments
///
/// * `received_pkt: &Vec<u8>` - Input data in wake format
///
/// # Output
///
/// * `Result<(u8, Vec<u8>), &str>` - command, data or error string
///
/// # Example
///
/// ```
/// extern crate wake;
///
/// let encoded_packet = vec![0xC0, 0x03, 0x05, 1, 2, 3, 4, 5, 0x6b];
/// let decoded_packet = wake::decode_packet(&encoded_packet);
/// match decoded_packet {
///     Ok(w) => {
///         println!("Decoded packet\t: {}", w);
///     },
///     Err(err) => println!("Error: {:?}", err),
/// }
/// ```
///
pub fn decode_packet(received_pkt: &Vec<u8>) -> Result<Packet, &str> {
    if received_pkt.len() < PACKET_MIN_LEN {
        return Err(TOO_SHORT_PACKET);
    }
    if received_pkt[0] != FEND {
        return Err(CANNOT_FIND_START);
    }
    let destuffed_pkt = destuffing(&received_pkt);
    if destuffed_pkt == None {
        return Err(DESTUFFING_FAILED);
    }
    let mut destuffed_pkt = destuffed_pkt.unwrap();
    let mut decoded = Packet::default();
    if (destuffed_pkt[1] & ADDR_MASK) != 0 {
        decoded.address = Some(destuffed_pkt[1] & !ADDR_MASK);
        destuffed_pkt.remove(1); // remove address from array
    } else {
        decoded.address = None;
    }
    let received_crc = *destuffed_pkt.last().unwrap();
    let destuffed_pkt_wo_crc = &destuffed_pkt[..destuffed_pkt.len() - 1]; // remove crc from packet
    let data_len = destuffed_pkt[2];
    if (destuffed_pkt_wo_crc.len() - 3) != data_len as usize {
        return Err(WRONG_LEN);
    }
    decoded.command = destuffed_pkt[1];
    decoded.data = if data_len != 0 {
        Some(destuffed_pkt_wo_crc[3..].to_vec())
    } else {
        None
    };
    if received_crc != crc(&destuffed_pkt_wo_crc.to_vec()) {
        return Err(WRONG_CRC);
    }
    Ok(decoded)
}

#[test]
fn decode_packet_wo_address_test() {
    let command = 0x03u8;
    let data = [1, 2, 3, 4, 5];
    let n = data.len() as u8;
    let crc = [0x6B];
    let wrong_crc = [0x6C];

    let mut good_packet = vec![FEND, command, n];
    good_packet.extend_from_slice(&data);
    good_packet.extend_from_slice(&crc);
    let decoded = decode_packet(&good_packet).unwrap();
    assert_eq!(decoded.command, command);
    assert_eq!(decoded.data.unwrap(), data);

    let bad_packet_too_short = vec![FEND, command, n];
    let decoded = decode_packet(&bad_packet_too_short);
    assert!(decoded.is_err(), TOO_SHORT_PACKET);

    let mut bad_packet_wo_start = vec![command, n];
    bad_packet_wo_start.extend_from_slice(&data);
    bad_packet_wo_start.extend_from_slice(&crc);
    let decoded = decode_packet(&bad_packet_wo_start);
    assert!(decoded.is_err(), CANNOT_FIND_START);

    let bad_packet_wrong_stuffing = vec![FEND, FESC, FESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed packed with wrong 3rd byte
    let decoded = decode_packet(&bad_packet_wrong_stuffing);
    assert!(decoded.is_err(), DESTUFFING_FAILED);

    let mut bad_packet_wrong_data_len = vec![FEND, command, n - 1];
    bad_packet_wrong_data_len.extend_from_slice(&data);
    bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
    let decoded = decode_packet(&bad_packet_wrong_data_len);
    assert!(decoded.is_err(), WRONG_LEN);

    let mut bad_packet_wrong_data_len = vec![FEND, command, n + 1];
    bad_packet_wrong_data_len.extend_from_slice(&data);
    bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
    let decoded = decode_packet(&bad_packet_wrong_data_len);
    assert!(decoded.is_err(), WRONG_LEN);

    let mut bad_packet_wrong_crc = vec![FEND, command, n];
    bad_packet_wrong_crc.extend_from_slice(&data);
    bad_packet_wrong_crc.extend_from_slice(&wrong_crc);
    let decoded = decode_packet(&bad_packet_wrong_crc);
    assert!(decoded.is_err(), WRONG_CRC);
}

#[test]
fn decode_packet_w_address_test() {
    let address = 0x09u8;
    let command = 0x03u8;
    let data = [1, 2, 3, 4, 5];
    let n = data.len() as u8;
    let crc = [0x6B];

    let mut good_packet = vec![FEND, address | 0x80u8, command, n];
    good_packet.extend_from_slice(&data);
    good_packet.extend_from_slice(&crc);
    let decoded = decode_packet(&good_packet).unwrap();
    assert_eq!(decoded.address.unwrap(), address);
    assert_eq!(decoded.command, command);
    assert_eq!(decoded.data.unwrap(), data);
}
