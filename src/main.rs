#![crate_name = "wake_lib"]
//! Wake protocol library

extern crate serialport;

const CRC_INIT: u8 = 0xDE;
const FEND: u8     = 0xC0;
const FESC: u8     = 0xDB;
const TFEND: u8    = 0xDC;
const TFESC: u8    = 0xDD;

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

/// Main function doc string
fn main() {
    println!("Hello, world!");
    let ports = serialport::available_ports();
    for p in ports.iter(){
        println!("{:?}", p);
        println!("p");
    }
    let xs: [u8; 5] = [1, 2, 3, 4, 5];

    let crc = do_crc_array(&xs);
    println!("CRC: 0x{:X}", crc);

    let mut v = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND];
    for x in &v {
        print!("{:02X} ", x);
    }

    do_stuffing(&mut v);
    println!("");

    for x in &v {
        print!("{:02X} ", x);
    }

    do_destuffing(&mut v);
    println!("");

    for x in &v {
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
}
