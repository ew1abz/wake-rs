#![crate_name = "wake"]
//! Wake protocol library

const CRC_INIT: u8 = 0xDE;
const FEND: u8     = 0xC0;
const FESC: u8     = 0xDB;
const TFEND: u8    = 0xDC;
const TFESC: u8    = 0xDD;
const PACKET_MIN_LEN: usize = 4;

const TOO_SHORT_PACKET:  &'static str = "Too short packet";
const CANNOT_FIND_START: &'static str = "Cannot find start of packet";
const DESTUFFING_FAILED: &'static str = "Destuffing failed";
const WRONG_LEN:         &'static str = "Wrong packet lenght";
const WRONG_CRC:         &'static str = "Wrong CRC";

pub struct WakePacket {
    pub cmd : u8,
    pub n   : u8,
    pub data: Vec<u8>,
    pub crc : u8,
}

/// Update CRC sum
///
/// # Arguments
///
/// * `crc` - A preinitialized crc
/// * `data` - A new data
///
/// # Example
///
/// ```
/// let mut crc: u8 = 0xDE;
/// wake::do_crc8(&mut crc, 0x31);
/// ```
pub fn do_crc8(crc: &mut u8, data: u8)
{
    let mut b = data;
    for _ in 0..8 {
        *crc = if (b ^ *crc) & 1 == 1 { ((*crc ^ 0x18) >> 1) | 0x80 } else { (*crc >> 1) & !0x80 };
        b = b >> 1;
    }
}

pub fn do_crc_vec(data: &Vec<u8>) -> u8 {
    let mut crc: u8 = CRC_INIT;
    for n in data {
        do_crc8(&mut crc, *n);
    }
    crc
}

/// Does byte stuffing in Vec
///
/// # Arguments
///
/// * `data: &Vec<u8>` - input data
pub fn stuffing(data: &Vec<u8>) -> Vec<u8> {
    let mut stuffed = vec![data[0]];
    for x in &data[1..] {
        match *x {
            FESC => { stuffed.push(FESC); stuffed.push(TFESC); },
            FEND => { stuffed.push(FESC); stuffed.push(TFEND); },
            _    =>   stuffed.push(*x),
        }
    }
    stuffed
}

/// Does byte destuffing in Vec
///
/// # Arguments
///
/// * `data` - Input data
/// * `Option<Vec<u8>>` - Output data wraped in Option
pub fn destuffing(data: &Vec<u8>) -> Option<Vec<u8>> {
    let mut output: Vec<u8> = vec![];
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            FESC => {
                if i > (data.len() - 2) {
                    return None;
                    }
                match data[i + 1] {
                    TFESC => { output.push(FESC); i += 1; },
                    TFEND => { output.push(FEND); i += 1; },
                    _     => return None,
                }                
            } 
            _ => output.push(data[i]), 
        }
        i += 1;
    }
    Some(output)
}

/// Encode packet to wake format
///
/// # Arguments
///
/// * `cmd` - Command code
/// * `data` - Data for transfer
///
/// # Output
///
/// * `Vec<u8>` - Encoded data in wake format
///
/// # Example
///
/// ```
/// let mut wake_packet: Vec<u8> = wake::encode_packet(0x03, &[1, 2, 3, 4, 5]);
/// ```
/// TODO: Insert address
pub fn encode_packet(cmd: u8, data: &[u8]) -> Vec<u8>
{
    let mut encoded_packet = vec![FEND, cmd, data.len() as u8];
    encoded_packet.extend(data.iter().cloned());
    let crc = do_crc_vec(&encoded_packet);
    encoded_packet.push(crc);
    stuffing(&encoded_packet)
}

pub fn decode_packet(recieved_pkt: &Vec<u8>) -> Result<WakePacket, &str> {
    if recieved_pkt.len() < PACKET_MIN_LEN {
         return Err(TOO_SHORT_PACKET)
    }
    if recieved_pkt[0] != FEND {
         return Err(CANNOT_FIND_START)
    }
    let destuffed_pkt = destuffing(&recieved_pkt);
    if destuffed_pkt == None {
         return Err(DESTUFFING_FAILED)
    }
    let destuffed_pkt = destuffed_pkt.unwrap();        
    let received_crc = *destuffed_pkt.last().unwrap();
    let destuffed_pkt_wo_crc = &destuffed_pkt[..destuffed_pkt.len() - 1]; // remove crc from packet       
    if (destuffed_pkt_wo_crc.len() - 3) != destuffed_pkt[2] as usize {
        return Err(WRONG_LEN)                   
    }
    if received_crc != do_crc_vec(&destuffed_pkt_wo_crc.to_vec()) {
        return Err(WRONG_CRC);           
    }
    Ok(WakePacket { cmd: destuffed_pkt[1],
                    n: destuffed_pkt[2], 
                    data: destuffed_pkt_wo_crc[3..].to_vec(), 
                    crc: received_crc})
}

#[cfg(test)]
mod tests {
    #[test]
    fn crc8_test() {
        const CRC_INIT: u8 = 0xDE;
        let mut crc: u8 = CRC_INIT;
        super::do_crc8(&mut crc, 0x00);
        assert!(crc == 0x48);
        super::do_crc8(&mut crc, 0x01);
        assert!(crc == 0xda);
        super::do_crc8(&mut crc, 0xff);
        assert!(crc == 0x1c);
        super::do_crc8(&mut crc, 0x55);
        assert!(crc == 0xda);
    }

    #[test]
    fn crc_vec_test() {
        let xs = vec![1, 2, 3, 4, 5];
        assert!(super::do_crc_vec(&xs) == 0xd6);
        let xs = vec![0xc0, 0x03, 0x00];
        assert!(super::do_crc_vec(&xs) == 0xeb);
    }

    #[test]
    fn stuffing_test() {
        let a = vec![super::FEND, super::FESC,               1, 2, 3, 4, 5, super::FEND];               // initial_data
        let b = vec![super::FEND, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed_data
        assert_eq!(super::stuffing(&a), b);
    }
       
    #[test]
    fn destuffing_test() {
        let t0 = vec![];                                                                                  // empty
        let t1 = vec![0x34];                                                                              // 1 byte 
        let t2 = vec![                                        1, 2, 3, 4, 5, super::FEND];                // stuffed data without first FEND
        let t3 = vec![super::FEND, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC];                // stuffed data without last byte
        let t4 = vec![super::FEND, super::FESC,               1, 2, 3, 4, 5, super::FESC, super::TFEND];  // stuffed data with missed 3rd byte
        let t5 = vec![super::FEND, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND];  // well stuffed data 
        let a5 = vec![super::FEND, super::FESC,               1, 2, 3, 4, 5, super::FEND];                // destuffed t5
        assert_eq!(super::destuffing(&t0), Some(vec![]));
        assert_eq!(super::destuffing(&t1), Some(t1));
        assert_eq!(super::destuffing(&t2), Some(t2));
        assert_eq!(super::destuffing(&t3), None);
        assert_eq!(super::destuffing(&t4), None);
        assert_eq!(super::destuffing(&t5), Some(a5));
    }

    #[test]
    fn encode_packet_test() {
        assert_eq!(super::encode_packet(0x03, &[1, 2, 3, 4, 5]), vec![super::FEND, 0x03, 0x05, 1, 2, 3, 4, 5, 0x6b]);
    }

    #[test]
    fn decode_packet_test() {
        let cmd = 0x03u8;
        let data = [1, 2, 3, 4, 5];
        let n = data.len() as u8;
        let crc = [0x6B];
        let wrong_crc = [0x6C];

        let mut good_packet = vec![super::FEND, cmd, n];
        good_packet.extend_from_slice(&data);
        good_packet.extend_from_slice(&crc);
        match super::decode_packet(&good_packet) {
            Ok(w) => { assert_eq!(w.cmd, cmd); assert_eq!(w.n, n); assert_eq!(w.data, data); },
            Err(err) => panic!("Error: {:?}", err),
        }

        let bad_packet_too_short = vec![super::FEND, cmd, n];
        match super::decode_packet(&bad_packet_too_short) {
            Ok(_w) => panic!("It should be Error"),
            Err(err) => assert_eq!(err, super::TOO_SHORT_PACKET),
        }

        let mut bad_packet_wo_start = vec![cmd, n];
        bad_packet_wo_start.extend_from_slice(&data);
        bad_packet_wo_start.extend_from_slice(&crc);
        match super::decode_packet(&bad_packet_wo_start) {
            Ok(_w) => panic!("It should be Error"),
            Err(err) => assert_eq!(err, super::CANNOT_FIND_START),
        }
        
        let bad_packet_wrong_stuffing = vec![super::FEND, super::FESC, super::FESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed packed with wrong 3rd byte
        match super::decode_packet(&bad_packet_wrong_stuffing) {
            Ok(_w) => panic!("It should be Error"),
            Err(err) => assert_eq!(err, super::DESTUFFING_FAILED),
        }

        let mut bad_packet_wrong_data_len = vec![super::FEND, cmd, n - 1];
        bad_packet_wrong_data_len.extend_from_slice(&data);
        bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
        match super::decode_packet(&bad_packet_wrong_data_len) {
            Ok(_w) => panic!("It should be Error"),
            Err(err) => assert_eq!(err, super::WRONG_LEN),
        }

        let mut bad_packet_wrong_data_len = vec![super::FEND, cmd, n + 1];
        bad_packet_wrong_data_len.extend_from_slice(&data);
        bad_packet_wrong_data_len.extend_from_slice(&wrong_crc);
        match super::decode_packet(&bad_packet_wrong_data_len) {
            Ok(_w) => panic!("It should be Error"),
            Err(err) => assert_eq!(err, super::WRONG_LEN),
        }

        let mut bad_packet_wrong_crc = vec![super::FEND, cmd, n];
        bad_packet_wrong_crc.extend_from_slice(&data);
        bad_packet_wrong_crc.extend_from_slice(&wrong_crc);
        match super::decode_packet(&bad_packet_wrong_crc) {
            Ok(_w) => panic!("It should be Error"),
            Err(err) => assert_eq!(err, super::WRONG_CRC),
        }
    }
}
