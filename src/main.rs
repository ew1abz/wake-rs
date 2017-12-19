#![crate_name = "wake"]
//! Wake protocol library

const CRC_INIT: u8 = 0xDE;
const FEND: u8     = 0xC0;
const FESC: u8     = 0xDB;
const TFEND: u8    = 0xDC;
const TFESC: u8    = 0xDD;
const PACKET_MIN_LEN: usize = 4;

struct WakePacket {
    cmd : u8,
    n   : u8,
    data: Vec<u8>,
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
fn encode_packet(cmd: u8, data: &[u8]) -> Vec<u8>
{
    let mut encoded_packet = vec![FEND, cmd, data.len() as u8];
    encoded_packet.extend(data.iter().cloned());
    let crc = do_crc_array(&encoded_packet);
    encoded_packet.push(crc);
    stuffing(encoded_packet)
}

fn decode_packet(recieved_pkt: &Vec<u8>) -> (Option<WakePacket>, String) {
    if recieved_pkt.len() < PACKET_MIN_LEN {
         return (None, format!("Too short packet"))
    }
    if recieved_pkt[0] != FEND {
         return (None, format!("Can't find start of packet"))
    }
    let destuffed_pkt = destuffing(&recieved_pkt);
    if destuffed_pkt == None {
         return (None, format!("Destuffing failed"))
    }
    let destuffed_pkt = destuffed_pkt.unwrap();        
    let received_crc = *destuffed_pkt.last().unwrap();
    let destuffed_pkt_wo_crc = &destuffed_pkt[..destuffed_pkt.len() - 2]; // remove crc from packet       
    
    if received_crc != do_crc_array(destuffed_pkt_wo_crc) {
        return (Option::None, format!("Wrong CRC"))           
    }
    (Option::Some(WakePacket { cmd: destuffed_pkt[1], n: destuffed_pkt[2], data: destuffed_pkt_wo_crc.to_vec() }), format!("No errors"))
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
    for _i in 0..8 {
        *crc = if (b ^ *crc) & 1 == 1 { ((*crc ^ 0x18) >> 1) | 0x80 } else { (*crc >> 1) & !0x80 };
        b = b >> 1;
    }
}

pub fn do_crc_array(arr: &[u8]) -> u8 {
    let mut crc: u8 = CRC_INIT;
    for n in arr.iter() {
        do_crc8(&mut crc, *n);
    }
    crc
}

pub fn do_crc_vec(arr: &Vec<u8>) -> u8 {
    let mut crc: u8 = CRC_INIT;
    for n in arr.iter() {
        do_crc8(&mut crc, *n);
    }
    crc
}

/// Does byte stuffing in Vec
///
/// # Arguments
///
/// * `data: &mut Vec<u8>` - Data buffer
pub fn stuffing(data: Vec<u8>) -> Vec<u8> {
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
    if data.len() < PACKET_MIN_LEN || data.len() < PACKET_MIN_LEN || data[0] != FESC {
         return None
    }
    let mut output = vec![data[0]];
    let mut i = 1; // skip the first element
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

/// Main function doc string
fn main() {
}

#[cfg(test)]
mod tests {
    #[test]
    fn do_crc8() {
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
    fn do_crc_array() {
        let xs: [u8; 5] = [1, 2, 3, 4, 5];
        assert!(super::do_crc_array(&xs) == 0xd6);
        let xs: [u8; 3] = [0xc0, 0x03, 0x00];
        assert!(super::do_crc_array(&xs) == 0xeb);
    }

    #[test]
    fn do_stuffing_v() {
        let     a = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FEND];               // initial_data
        let mut b = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed_data
        let mut aa = a.clone();
        assert_eq!(super::do_stuffing_v(a), b);
    }

    // #[test]
    // fn do_stuffing_destuffing() {
    //     let     a = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FEND];               // initial_data
    //     let mut b = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed_data
    //     let mut c = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC];               // stuffed_data without last byte
    //     let mut d = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed_data with missed 3rd byte

    //     let mut aa = a.clone();
    //     super::do_stuffing(&mut aa);
    //     assert_eq!(aa, b);

    //     assert!(super::do_destuffing(&mut b) == true);
    //     assert_eq!(a, b);

    //     assert!(super::do_destuffing(&mut c) == false);
    //     assert!(super::do_destuffing(&mut d) == false);
    // }
        
    #[test]
    fn do_stuffing_destuffing() {
        let t1 = vec![];                                                                                  // empty
        let t2 = vec![                                        1, 2, 3, 4, 5, super::FEND];                // w/o first FESC
        let t3 = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC];                // stuffed_data without last byte
        let t4 = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FESC, super::TFEND];  // stuffed_data with missed 3rd byte
        let t5 = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND];  // good stuffed_data 
        let a5 = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FEND];                // destuffed t5
        assert_eq!(super::destuffing(&t1), None);
        assert_eq!(super::destuffing(&t2), None);
        assert_eq!(super::destuffing(&t3), None);
        assert_eq!(super::destuffing(&t4), None);
        assert_eq!(super::destuffing(&t5), Some(a5));
    }

    #[test]
    fn encode_packet() {
    assert_eq!(super::encode_packet(0x03, &[1, 2, 3, 4, 5]), vec![super::FEND, 0x03, 0x05, 1, 2, 3, 4, 5, 0x6b]);
    }
}
