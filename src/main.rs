#![crate_name = "wake_lib"]
//! Wake protocol library

extern crate serialport;

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

fn encode_packet(cmd: u8, data: &[u8]) -> Vec<u8>
{
    let mut encoded_packet = vec![FEND, cmd, data.len() as u8];
    encoded_packet.extend(data.iter().cloned());
    let crc = do_crc_array(&encoded_packet);
    encoded_packet.push(crc);
    do_stuffing(&mut encoded_packet);
    encoded_packet
}

fn encode_packet_v(cmd: u8, data: &[u8]) -> Vec<u8>
{
    let mut encoded_packet = vec![FEND, cmd, data.len() as u8];
    encoded_packet.extend(data.iter().cloned());
    let crc = do_crc_array(&encoded_packet);
    encoded_packet.push(crc);
    do_stuffing_v(encoded_packet)
}

fn decode_packet(recieved_pkt: &Vec<u8>) -> (Option<WakePacket>, String) {
    if recieved_pkt.len() < PACKET_MIN_LEN {
         return (None, format!("Too short packet"))
    }
    if recieved_pkt[0] != FEND {
         return (None, format!("Can't find start of packet"))
    }
    let destuffed_pkt = do_destuffing_o(&recieved_pkt);
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

///    Does byte stuffing in Vec
///    \param data: &mut Vec<u8> Data buffer
/// Warning "for i in 0..v.len()"" doesn't work here. Is is not update the len() of vector
pub fn do_stuffing(data: &mut Vec<u8>) {
    let mut i = 1; // skip the first element
    while i < data.len() {
        match data[i] {
            FESC => data.insert(i + 1, TFESC),
            FEND => { data[i] = FESC; data.insert(i + 1, TFEND); },
            _    => (),
        }
        i += 1;
    }
}

pub fn do_stuffing_v(data: Vec<u8>) -> Vec<u8> {
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

///    Does byte destuffing in Vec
///    \param data: &mut Vec<u8> Data buffer
pub fn do_destuffing(data: &mut Vec<u8>) -> bool {
    let mut i = 1; // skip the first element
    while i < data.len() {
        if data[i] == FESC {
            if i > (data.len() - 2) {
                 return false;
                 }
            match data[i + 1] {
                TFESC => {data.remove(i + 1);},
                TFEND => { data[i] = FEND; data.remove(i + 1);},
                _     => return false,
            }
        }
        i += 1;
    }
    true
}

/// Does byte destuffing in Vec
/// # Arguments
///
/// * `crc` - A preinitialized crc
/// * `data` - Input data
pub fn do_destuffing_o(data: &Vec<u8>) -> Option<Vec<u8>> {
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
    let ports = serialport::available_ports();
    for p in ports.iter(){
        println!("{:?}", p);
    }

    let xs: [u8; 5] = [1, 2, 3, 4, 5];
    let crc = do_crc_array(&xs);
    println!("CRC: 0x{:X}", crc);

    let mut v = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND];
    print!("\nTest packet: ");
    for x in &v {
        print!("{:02X} ", x);
    }

    do_stuffing(&mut v);
    print!("\nStuffed packet: ");
    for x in &v {
        print!("{:02X} ", x);
    }

    do_destuffing(&mut v);
    print!("\nDestuffed packet: ");
    for x in &v {
        print!("{:02X} ", x);
    }
    
    let tx = encode_packet(0x03, &xs);
    print!("\nEncoded packet: ");
    for x in &tx {
        print!("{:02X} ", x);
    }
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

    #[test]
    fn do_stuffing_destuffing() {
        let     a = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FEND];               // initial_data
        let mut b = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed_data
        let mut c = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC];               // stuffed_data without last byte
        let mut d = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FESC, super::TFEND]; // stuffed_data with missed 3rd byte

        let mut aa = a.clone();
        super::do_stuffing(&mut aa);
        assert_eq!(aa, b);

        assert!(super::do_destuffing(&mut b) == true);
        assert_eq!(a, b);

        assert!(super::do_destuffing(&mut c) == false);
        assert!(super::do_destuffing(&mut d) == false);
    }
        
    #[test]
    fn do_stuffing_destuffing_o() {
        let t1 = vec![];                                                                                  // empty
        let t2 = vec![                                        1, 2, 3, 4, 5, super::FEND];                // w/o first FESC
        let t3 = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC];                // stuffed_data without last byte
        let t4 = vec![super::FESC, super::FESC,               1, 2, 3, 4, 5, super::FESC, super::TFEND];  // stuffed_data with missed 3rd byte
        let t5 = vec![super::FESC, super::FESC, super::TFESC, 1, 2, 3, 4, 5, super::FESC, super::TFEND];  // good stuffed_data 
        assert!(super::do_destuffing_o(&t1 == None));
    }

    #[test]
    fn encode_packet() {
    assert_eq!(  super::encode_packet(0x03, &[1, 2, 3, 4, 5]), vec![super::FEND, 0x03, 0x05, 1, 2, 3, 4, 5, 0x6b]);
    assert_eq!(super::encode_packet_v(0x03, &[1, 2, 3, 4, 5]), vec![super::FEND, 0x03, 0x05, 1, 2, 3, 4, 5, 0x6b]);
    }
}
