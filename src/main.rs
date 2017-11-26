#![crate_name = "wake_lib"]
//! Wake protocol library

extern crate serialport;

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
///let mut crc: u8 = 0xDE;
///wake::do_crc8(&mut crc, 0x31);
/// ```
pub fn do_crc8(crc: &mut u8, _b: u8)
{
    let mut b = _b;
    for _i in 0..8 {
        if (b ^ *crc) & 1 == 1 {
            *crc = ((*crc ^ 0x18) >> 1) | 0x80;
        } else {
            *crc = (*crc >> 1) & !0x80;
        }
        b = b >> 1;
    }
}

pub fn do_crc_array(arr: &[u8]) -> u8 {
    const CRC_INIT: u8 = 0xDE;
    let mut crc: u8 = CRC_INIT;
    for n in arr.iter() {
        do_crc8(&mut crc, *n);
    }
    crc
}

///    Do byte stuffing in buffer and update pointer, if needed
///    \param b Byte to tx
///    \param dptr Data pointer
///    \param buff Buffer
// static void byte_stuff(unsigned char b, int &bptr, char *buff)
// {
//     if ((b == FEND) || (b == FESC))
//     {
//         buff[bptr++] = FESC;
//         buff[bptr++] = (b == FEND)? TFEND : TFESC;
//     }
//     else buff[bptr++] = b;
// }


/// Main function doc string
fn main() {
    println!("Hello, world!");
    let ports = serialport::available_ports();
    for p in ports.iter(){
        println!("{:?}", p);
        println!("p");
    }
    let xs: [u8; 5] = [1, 2, 3, 4, 5];
    //for (i, n) in xs.iter().enumerate(){
    //    println!("{} element of the array: {}", i, n);
    //}
    let crc = do_crc_array(&xs);
    println!("CRC: 0x{:X}", crc);
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
        //panic!("Make this test fail {:X}", crc);
    }

    #[test]
    fn do_crc_array() {
        let xs: [u8; 5] = [1, 2, 3, 4, 5];
        assert!(super::do_crc_array(&xs) == 0xd6);
    }
    //#[test]
    //fn another() {
    //    panic!("Make this test fail");
   // }
}