#![crate_name = "wake"]
//! Wake protocol library

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
pub const DATA_MAX_LEN: usize = 0xff;

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
// fn crc(data: &Vec<u8>) -> u8 {
//     let mut crc: u8 = CRC_INIT;
//     for n in data {
//         crc8(&mut crc, *n);
//     }
//     crc
// }

pub trait Decode {
    fn decode(&self) -> Result<Packet, &str>;
}

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
}

trait Wake {
    // TODO: change the name?
    fn crc(&self) -> u8;
    fn stuff(&self) -> Vec<u8>;
    fn dry(&self) -> Result<Vec<u8>, &str>;
}

impl Wake for Vec<u8> {
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
        let mut stuffed = vec![self[0]];
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
    /// * `Result<Vec<u8>>` - Destuffed data wrapped in Result
    ///
    //fn dry(&self) -> Result<Vec<u8>> {
    fn dry(&self) -> Result<Vec<u8>, &str> {
        let mut output: Vec<u8> = vec![];
        let mut i = 0;
        while i < self.len() {
            match self[i] {
                FESC => {
                    if i > (self.len() - 2) {
                        return Err(WRONG_LEN);
                    }
                    output.push(match self[i + 1] {
                        TFESC => FESC,
                        TFEND => FEND,
                        _ => return Err(DESTUFFING_FAILED),
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

impl Decode for Vec<u8> {
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
    /// use wake::Decode;
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
    fn decode(&self) -> Result<Packet, &str> {
        // 1: Check packet length
        if self.len() < PACKET_MIN_LEN {
            return Err(TOO_SHORT_PACKET);
        }
        // 2: Check START symbol (FEND)
        if self[0] != FEND {
            return Err(CANNOT_FIND_START);
        }
        // 3: Dry packet (remove stuffed bytes)
        let mut destuffed_pkt = self.dry()?;
        let mut v_iter = destuffed_pkt.iter().enumerate();
        v_iter.next(); // skip start symbol
                       // 4: Get an address (if exists) and a command
        let mut decoded = Packet::default();
        let (_, d) = v_iter.next().unwrap();
        match d {
            addr @ ADDR_MASK..=0xff => {
                decoded.address = Some(addr & !ADDR_MASK);
                let (_, cmd) = v_iter.next().unwrap();
                decoded.command = *cmd;
            }
            cmd @ _ => {
                decoded.address = None;
                decoded.command = *cmd;
            }
        };
        // 5: Get data length
        let (i, data_len) = v_iter.next().unwrap();
        // 8: Check data length
        if (destuffed_pkt.len() - i - 2) != *data_len as usize {
            return Err(WRONG_LEN);
        }
        // 9: Get data
        decoded.data = match data_len {
            0 => None,
            _ => Some(destuffed_pkt[i + 1..destuffed_pkt.len() - 1].to_vec()),
        };
        // 6: Get CRC and remove it
        let received_crc = destuffed_pkt.remove(destuffed_pkt.len() - 1);
        // 10: Check CRC
        // print!(
        //     "received_crc = {:02X} destuffed_pkt.to_vec().crc() = {:02X} pkt = {:?}",
        //     received_crc,
        //     destuffed_pkt.to_vec().crc(),
        //     destuffed_pkt.to_vec()
        // );
        return if received_crc != destuffed_pkt.to_vec().crc() {
            Err(WRONG_CRC)
        } else {
            Ok(decoded)
        };
    }
}

impl Encode for Packet {
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
    /// use wake::Encode;
    ///
    /// let p = wake::Packet{address: Some(0x12), command: 3, data: Some(vec!{0x00, 0xeb})};
    /// let mut encoded_packet: Vec<u8> = p.encode();
    /// ```
    ///
    fn encode(&self) -> Vec<u8> {
        let mut encoded_packet = vec![];
        // 1. FEND
        encoded_packet.push(FEND);
        // 2. Address, if exists
        if let Some(addr) = self.address {
            encoded_packet.push(addr as u8 | ADDR_MASK);
        }
        // 3. Command
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
        encoded_packet.stuff()
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
    let a = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND]; // initial_data
    let b = vec![FEND, FESC, TFESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed_data
    assert_eq!(a.stuff(), b);
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
    assert_eq!(t3.dry(), Err(WRONG_LEN));
    assert_eq!(t4.dry(), Err(DESTUFFING_FAILED));
    assert_eq!(t5.dry(), Ok(a5));
}
#[test]
fn encode_packet_test() {
    let wp = Packet {
        address: Some(0x12),
        command: 3,
        data: Some(vec![0x00, 0xeb]),
    };
    assert_eq!(wp.encode(), vec![FEND, 0x92, 0x03, 0x02, 0x00, 0xeb, 114]);
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
    assert_eq!(decoded.err(), Some(TOO_SHORT_PACKET));

    let mut bad_packet_wo_start = vec![command, n];
    bad_packet_wo_start.extend_from_slice(&data);
    bad_packet_wo_start.extend_from_slice(&crc);
    let decoded = bad_packet_wo_start.decode();
    assert_eq!(decoded.err(), Some(CANNOT_FIND_START));

    let bad_packet_wrong_stuffing = vec![FEND, FESC, FESC, 1, 2, 3, 4, 5, FESC, TFEND]; // stuffed packed with wrong 3rd byte
    let decoded = bad_packet_wrong_stuffing.decode();
    assert_eq!(decoded.err(), Some(DESTUFFING_FAILED));

    let mut bad_packet_wrong_data_len = vec![FEND, command, n - 1];
    bad_packet_wrong_data_len.extend_from_slice(&data);
    bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
    let decoded = bad_packet_wrong_data_len.decode();
    assert_eq!(decoded.err(), Some(WRONG_LEN));

    let mut bad_packet_wrong_data_len = vec![FEND, command, n + 1];
    bad_packet_wrong_data_len.extend_from_slice(&data);
    bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
    let decoded = bad_packet_wrong_data_len.decode();
    assert_eq!(decoded.err(), Some(WRONG_LEN));

    let mut bad_packet_wrong_crc = vec![FEND, command, n];
    bad_packet_wrong_crc.extend_from_slice(&data);
    bad_packet_wrong_crc.extend_from_slice(&wrong_crc);
    let decoded = bad_packet_wrong_crc.decode();
    assert_eq!(decoded.err(), Some(WRONG_CRC));
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
}

#[test]
fn random_encode_decode_test() {
    let mut rng = rand::thread_rng();

    for _ in 0..100_000 {
        let address_exists = rng.gen_bool(0.5);
        let n = rng.gen_range(0, 0x100);
        let mut d: Vec<u8> = Vec::new();
        for _ in 0..n {
            d.push(rng.gen_range(0, 0xff));
        }

        let wp = Packet {
            address: if address_exists {
                Some(rng.gen_range(0, 0x7f))
            } else {
                None
            },
            command: rng.gen_range(0, 0x7f),
            data: if d.len() == 0 { None } else { Some(d.clone()) },
        };
        // print!("{}\n", &wp);
        let encoded = wp.encode();
        let decoded = encoded.decode().unwrap();
        assert_eq!(decoded.address, wp.address);
        assert_eq!(decoded.command, wp.command);
        assert_eq!(decoded.data, wp.data);
    }
}
