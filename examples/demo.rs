extern crate wake;

use wake::*;

const FEND:  u8    = 0xC0;
const FESC:  u8    = 0xDB;

fn print_packet(header: &str, v: &Vec<u8>) {
    print!("\n{}:\t", header);
    for x in v {
        print!("{:02X} ", x);
    }
}

/// Main function doc string
fn main() {
    let xs: [u8; 5] = [1, 2, 3, 4, 5];
    let crc = wake::do_crc_array(&xs);
    println!("CRC: 0x{:X}", crc);

    let v = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND];
    print_packet("Initial packet", &v);

    let stuffed = stuffing(&v);
    print_packet("Stuffed packet", &stuffed);

    let destuffed = destuffing(&stuffed);
    print_packet("Destuffed packet", &destuffed.unwrap());
    
    let encoded = encode_packet(0x03, &xs);
    print_packet("Encoded packet", &encoded);
   
    let decoded = decode_packet(&encoded);
    match decoded {
        Ok(w) => print!("Decoded packet\tcmd: 0x{:X}\tn: 0x{:X}", w.cmd, w.n),
        Err(err) => println!("Error: {:?}", err),
    }
    //let d = decoded.as_ref().map(|s| s.clone());
    //let d1 = d.clone();
    
//      Ok(w) => print!("Decoded packet\tcmd: 0x{:X}\tn: 0x{:X}", w.as_ref().unwrap().cmd, w.as_ref().unwrap().n),
    //print_packet("Decoded packet data\t", &decoded.unwrap().data);
}
