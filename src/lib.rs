#![crate_name = "wake_rs"]
//! `Wake` is a serial communication protocol highly optimized for microcontrollers.
//! `wake-rs` is a library written in Rust for encoding/decoding Wake protocol packets.

#[cfg(test)]
extern crate rand;

#[cfg(test)]
use rand::Rng;
use std::fmt;

const FEND: u8 = 0xC0;
const FESC: u8 = 0xDB;
const TFEND: u8 = 0xDC;
const TFESC: u8 = 0xDD;

const ADDR_MASK: u8 = 0x80;
const CRC_INIT: u8 = 0xDE;
const PACKET_MIN_LEN: usize = 4;

/// Maximum supported data length. Might be reduced depends on available resources.
pub const DATA_MAX_LEN: usize = 0xff;

/// Wake decoder/encoder errors
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WakeError {
    TooShortPacket,
    CannotFindStart,
    DestuffingFailed,
    WrongPacketLength,
    WrongPacketCrc,
    WrongAddrRange,
    WrongCmdRange,
}

impl std::error::Error for WakeError {
    fn description(&self) -> &str {
        match *self {
            WakeError::TooShortPacket => "Too short packet",
            WakeError::CannotFindStart => "Can't find a start of the packet",
            WakeError::DestuffingFailed => "De-stuffing failed",
            WakeError::WrongPacketLength => "Wrong packet length",
            WakeError::WrongPacketCrc => "Wrong packet CRC",
            WakeError::WrongAddrRange => "Address is out of range [0 - 127]",
            WakeError::WrongCmdRange => "Command is out of range [0 - 127]",
        }
    }
}

impl fmt::Display for WakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wake error: {:?}", self)
    }
}

/// Wake packet: address, command, and data
#[derive(Default)]
pub struct Packet {
    /// Device address (optional) [0 - 127]
    pub address: Option<u8>,
    /// Command [0 - 127]
    pub command: u8,
    /// Data load (optional)
    pub data: Option<Vec<u8>>,
}

impl fmt::Display for Packet {
    /// Show error in human readable format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let addr = match self.address {
            Some(a) => format!("ADDR: 0x{:02X}", a),
            None => "ADDR: ----".to_string(),
        };

        let cmd = format!("CMD:  0x{:02X}", self.command);

        let data = match &self.data {
            Some(d) => {
                let mut print = format!("DATA: {} bytes\n", d.len());
                print.push_str(&format!(
                    "     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f"
                ));
                for (i, item) in d.iter().enumerate() {
                    if (i == 0) | (i % 16 == 0) {
                        print.push_str(&format!("\n{:02x}: ", i));
                    }
                    print.push_str(&format!("{:02x} ", item));
                }
                print
            }
            None => format!("DATA: none"),
        };
        write!(f, "{}\n{}\n{}\n", addr, cmd, data)
    }
}

trait Wake {
    fn crc(&self) -> u8;
    fn stuff(&self) -> Vec<u8>;
    fn dry(&self) -> Result<Vec<u8>, WakeError>;
}

/// Calculate CRC sum of data in a vector
impl Wake for Vec<u8> {
    /// # Input
    ///
    /// * 'Vec<u8>` - input data
    ///
    /// # Output
    ///
    /// * `u8` - Calculated CRC
    ///
    fn crc(&self) -> u8 {
        let mut crc: u8 = CRC_INIT;

        let mut crc8 = |data| {
            let mut b = data;
            for _ in 0..8 {
                crc = if (b ^ crc) & 1 == 1 {
                    ((crc ^ 0x18) >> 1) | 0x80
                } else {
                    (crc >> 1) & !0x80
                };
                b = b >> 1;
            }
        };

        for n in self {
            crc8(*n);
        }
        crc
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
    fn stuff(&self) -> Vec<u8> {
        assert_eq!(self.len() >= (PACKET_MIN_LEN - 1), true); // without CRC
        assert_eq!(self[0], FEND);
        let mut stuffed: Vec<u8> = vec![self[0]];
        for x in &self[1..] {
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

    /// Byte destuffing in a vector
    /// /// Translate stuffed bytes into normal data
    ///
    /// # Arguments
    ///
    /// * `data` - Input data
    ///
    /// # Output
    ///
    /// * `Result<Vec<u8>, WakeError>` - Destuffed data wrapped in Result
    ///
    fn dry(&self) -> Result<Vec<u8>, WakeError> {
        let mut output: Vec<u8> = vec![];
        let mut i = 0;
        while i < self.len() {
            match self[i] {
                FESC => {
                    if i > (self.len() - 2) {
                        return Err(WakeError::WrongPacketLength);
                    }
                    output.push(match self[i + 1] {
                        TFESC => FESC,
                        TFEND => FEND,
                        _ => return Err(WakeError::DestuffingFailed),
                    });
                    i += 1;
                }
                _ => output.push(self[i]),
            }
            i += 1;
        }
        Ok(output)
    }
}

/// Decode data from wake format to wake packet structure
pub trait Decode {
    fn decode(&self) -> Result<Packet, WakeError>;
}

/// Decode Vec<u8> from wake format to wake packet structure
impl Decode for Vec<u8> {
    /// # Output
    ///
    /// * `Result<Packet, WakeError>` - command, data or error
    ///
    /// # Example
    ///
    /// ```
    /// extern crate wake_rs;
    /// use wake_rs::Decode;
    ///
    /// let encoded_packet = vec![0xC0, 0x03, 0x05, 1, 2, 3, 4, 5, 0x6b];
    /// let decoded_packet = encoded_packet.decode();
    /// match decoded_packet {
    ///     Ok(w) => {
    ///         println!("Decoded packet\t: {}", w);
    ///     },
    ///     Err(err) => println!("Error: {:?}", err),
    /// }
    /// ```
    ///
    fn decode(&self) -> Result<Packet, WakeError> {
        // 1: Check packet length
        if self.len() < PACKET_MIN_LEN {
            return Err(WakeError::TooShortPacket);
        }
        // 2: Check START symbol (FEND)
        if self[0] != FEND {
            return Err(WakeError::CannotFindStart);
        }
        // 3: Dry packet (remove stuffed bytes)
        let mut destuffed_pkt = self.dry()?;
        let mut v_iter = destuffed_pkt.iter().enumerate();
        v_iter.next(); // skip start symbol
                       // 4: Get an address (if exists) and a command
        let mut decoded = Packet::default();
        let (_, d) = v_iter.next().ok_or_else(|| WakeError::TooShortPacket)?;
        match d {
            addr @ ADDR_MASK..=0xff => {
                decoded.address = Some(addr & !ADDR_MASK);
                let (_, cmd) = v_iter.next().ok_or_else(|| WakeError::TooShortPacket)?;
                decoded.command = *cmd;
            }
            cmd @ _ => {
                decoded.address = None;
                decoded.command = *cmd;
            }
        };
        // 5: Get data length
        let (i, data_len) = v_iter.next().ok_or_else(|| WakeError::TooShortPacket)?;
        // 8: Check data length
        if (destuffed_pkt.len() - i - 2) != *data_len as usize {
            return Err(WakeError::WrongPacketLength);
        }
        // 9: Get data
        decoded.data = match data_len {
            0 => None,
            _ => Some(destuffed_pkt[i + 1..destuffed_pkt.len() - 1].to_vec()),
        };
        // 6: Get CRC and remove it
        let received_crc = destuffed_pkt.remove(destuffed_pkt.len() - 1);
        // 10: Check CRC
        if received_crc != destuffed_pkt.to_vec().crc() {
            Err(WakeError::WrongPacketCrc)
        } else {
            Ok(decoded)
        }
    }
}

/// Encode packet to wake format
pub trait Encode {
    fn encode(&self) -> Result<Vec<u8>, WakeError>;
}

/// Encode packet to wake format
impl Encode for Packet {
    /// # Input
    ///
    /// * Wake packet structure with address, command code and data. Address and data are optional.
    ///
    /// # Output
    ///
    /// * `Vec<u8>` - Encoded data in wake format
    ///
    /// # Example
    ///
    /// ```
    /// extern crate wake_rs;
    /// use wake_rs::Encode;
    ///
    /// let p = wake_rs::Packet{address: Some(0x12), command: 3, data: Some(vec!{0x00, 0xeb})};
    /// let encoded_packet: Vec<u8> = p.encode().unwrap();
    /// ```
    ///
    fn encode(&self) -> Result<Vec<u8>, WakeError> {
        let mut encoded_packet: Vec<u8> = vec![];
        // 1. FEND
        encoded_packet.push(FEND);
        // 2. Address, if exists
        if let Some(addr) = self.address {
            if addr > 0x7f {
                return Err(WakeError::WrongAddrRange);
            }
            encoded_packet.push(addr | ADDR_MASK);
        }
        // 3. Command
        if self.command > 0x7f {
            return Err(WakeError::WrongCmdRange);
        }
        encoded_packet.push(self.command);
        // 4. Data length; data, if exists
        match &self.data {
            Some(d) => {
                encoded_packet.push(d.len() as u8);
                encoded_packet.extend(d.iter().cloned());
            }
            None => encoded_packet.push(0),
        }
        // 5. CRC
        encoded_packet.push(encoded_packet.crc());
        // 6. Stuffing
        Ok(encoded_packet.stuff())
    }
}

#[test]
fn crc_test() {
    let xs = vec![1, 2, 3, 4, 5];
    assert_eq!(xs.crc(), 0xd6);

    let xs = vec![0xc0, 0x03, 0x00];
    assert_eq!(xs.crc(), 0xeb);

    let xs = vec![0xc0, 0x89, 0x03, 0x05, 1, 2, 3, 4, 5];
    assert_eq!(xs.crc(), 0x69);
}

#[test]
fn stuff_test() {
    // Regular packet
    let a = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND]; // initial_data
    let b = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed_data
    assert_eq!(a.stuff(), b);

    // packet with min len
    let a = vec![FEND, 3, 0];
    assert_eq!(a.stuff(), a);

    // empty packet, should panic
    let a = vec![];
    let result = std::panic::catch_unwind(|| a.stuff());
    assert!(result.is_err());

    // short packet, should panic
    let a = vec![FEND, 3];
    let result = std::panic::catch_unwind(|| a.stuff());
    assert!(result.is_err());
}

#[test]
fn dry_test() {
    let t0 = vec![]; // empty
    let t1 = vec![0x34]; // 1 byte
    let t2 = vec![1, 2, 3, 4, 5, FEND]; // stuffed data without first FEND
    let t3 = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC]; // stuffed data without last byte
    let t4 = vec![FEND, FESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed data with missed 3rd byte
    let t5 = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC, TFEND]; // well stuffed data
    let a5 = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND]; // destuffed t5
    assert_eq!(t0.dry(), Ok(vec![]));
    assert_eq!(t1.clone().dry(), Ok(t1));
    assert_eq!(t2.clone().dry(), Ok(t2));
    assert_eq!(t3.dry(), Err(WakeError::WrongPacketLength));
    assert_eq!(t4.dry(), Err(WakeError::DestuffingFailed));
    assert_eq!(t5.dry(), Ok(a5));
}
#[test]
fn encode_packet_test() {
    // address is out of range
    let wp = Packet {
        address: Some(128),
        command: 9,
        data: Some(vec![0x12, 0x34]),
    };
    assert_eq!(wp.encode(), Err(WakeError::WrongAddrRange));
    // command is out of range
    let wp = Packet {
        address: None,
        command: 128,
        data: Some(vec![0x12, 0x34]),
    };
    assert_eq!(wp.encode(), Err(WakeError::WrongCmdRange));
    // without address
    let wp = Packet {
        address: None,
        command: 9,
        data: Some(vec![0x12, 0x34]),
    };
    assert_eq!(wp.encode(), Ok(vec![FEND, 0x09, 0x02, 0x12, 0x34, 160]));
    // with data
    let wp = Packet {
        address: Some(0x12),
        command: 3,
        data: Some(vec![0x00, 0xeb]),
    };
    assert_eq!(
        wp.encode(),
        Ok(vec![FEND, 0x92, 0x03, 0x02, 0x00, 0xeb, 114])
    );
    // empty packet
    let wp = Packet {
        address: Some(0x13),
        command: 4,
        data: None,
    };
    assert_eq!(wp.encode(), Ok(vec![FEND, 0x93, 0x04, 0x00, 218]));
    // empty packet with stuffing
    let wp = Packet {
        address: Some(0x40),
        command: 0x40,
        data: None,
    };
    assert_eq!(wp.encode(), Ok(vec![FEND, FESC, TFEND, 0x40, 0x00, 229]));
}

#[test]
fn decode_wo_address_test() {
    let command = 0x03u8;
    let data = [1, 2, 3, 4, 5];
    let n = data.len() as u8;
    let crc = [0x6B];
    let wrong_crc = [0x6C];

    let mut good_packet = vec![FEND, command, n];
    good_packet.extend_from_slice(&data);
    good_packet.extend_from_slice(&crc);
    let decoded = good_packet.decode().unwrap(); // TODO: need unwrap?
    assert_eq!(decoded.command, command);
    assert_eq!(decoded.data.unwrap(), data);

    let bad_packet_too_short = vec![FEND, command, n];
    let decoded = bad_packet_too_short.decode();
    assert_eq!(decoded.err(), Some(WakeError::TooShortPacket));

    let mut bad_packet_wo_start = vec![command, n];
    bad_packet_wo_start.extend_from_slice(&data);
    bad_packet_wo_start.extend_from_slice(&crc);
    let decoded = bad_packet_wo_start.decode();
    assert_eq!(decoded.err(), Some(WakeError::CannotFindStart));

    let bad_packet_wrong_stuffing = vec![FEND, FESC, FESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed packed with wrong 3rd byte
    let decoded = bad_packet_wrong_stuffing.decode();
    assert_eq!(decoded.err(), Some(WakeError::DestuffingFailed));

    let mut bad_packet_wrong_data_len = vec![FEND, command, n - 1];
    bad_packet_wrong_data_len.extend_from_slice(&data);
    bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
    let decoded = bad_packet_wrong_data_len.decode();
    assert_eq!(decoded.err(), Some(WakeError::WrongPacketLength));

    let mut bad_packet_wrong_data_len = vec![FEND, command, n + 1];
    bad_packet_wrong_data_len.extend_from_slice(&data);
    bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
    let decoded = bad_packet_wrong_data_len.decode();
    assert_eq!(decoded.err(), Some(WakeError::WrongPacketLength));

    let mut bad_packet_wrong_crc = vec![FEND, command, n];
    bad_packet_wrong_crc.extend_from_slice(&data);
    bad_packet_wrong_crc.extend_from_slice(&wrong_crc);
    let decoded = bad_packet_wrong_crc.decode();
    assert_eq!(decoded.err(), Some(WakeError::WrongPacketCrc));
}

#[test]
fn decode_w_address_test() {
    let address = 0x09u8;
    let command = 0x03u8;
    let data = [1, 2, 3, 4, 5];
    let n = data.len() as u8;
    let crc = [0x69];

    let mut good_packet = vec![FEND, address | 0x80u8, command, n];
    good_packet.extend_from_slice(&data);
    good_packet.extend_from_slice(&crc);
    let decoded = good_packet.decode();
    assert_eq!(decoded.is_ok(), true);
    let decoded = decoded.unwrap();
    assert_eq!(decoded.address.unwrap(), address);
    assert_eq!(decoded.command, command);
    assert_eq!(decoded.data.unwrap(), data);

    // 0x40 test
    let good_packet = vec![FEND, FESC, TFEND, 0x40, 0x00, 229];
    let decoded = good_packet.decode();
    assert_eq!(decoded.is_ok(), true);
    let decoded = decoded.unwrap();
    assert_eq!(decoded.address.unwrap(), 0x40);
    assert_eq!(decoded.command, 0x40);
    assert_eq!(decoded.data, None);
}

#[test]
fn random_encode_decode_test() {
    let mut rng = rand::thread_rng();

    for _ in 0..100_000 {
        let address_exists = rng.gen_bool(0.5);
        let n = rng.gen_range(0..0x100);
        let mut d: Vec<u8> = Vec::new();
        for _ in 0..n {
            d.push(rng.gen_range(0..0xff));
        }

        let wp = Packet {
            address: if address_exists {
                Some(rng.gen_range(0..0x7f))
            } else {
                None
            },
            command: rng.gen_range(0..0x7f),
            data: if d.len() == 0 { None } else { Some(d.clone()) },
        };
        // print!("{}\n", &wp);
        let encoded = wp.encode().unwrap();
        let decoded = encoded.decode().unwrap();
        assert_eq!(decoded.address, wp.address);
        assert_eq!(decoded.command, wp.command);
        assert_eq!(decoded.data, wp.data);
    }
}
