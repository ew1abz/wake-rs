extern crate wake_rs;

use wake_rs::*;
//use wake::wake_constants;

// const FEND: u8 = 0xC0;
// const FESC: u8 = 0xDB;

fn print_packet(header: &str, v: &Vec<u8>) {
    print!("\n{}\t", header);
    for x in v {
        print!("{:02X} ", x);
    }
}

/// Main function doc string
fn main() {
    // let xs = vec![1, 2, 3, 4, 5];
    // let crc = wake::crc_vec(&xs);
    // println!("CRC: 0x{:X}", crc);

    // let v = vec![FEND, FESC, 1, 2, 3, 4, 5, FEND];
    // print_packet("Initial packet:\t", &v);

    // let stuffed = stuffing(&v);
    // print_packet("Stuffed packet:\t", &stuffed);

    // let destuffed = destuffing(&stuffed);
    // print_packet("Destuffed packet:", &destuffed.unwrap());

    //let encoded = wake::encode_packet(0x03, Some(&xs));
    let wp = wake::Packet{addr: Some(0x12), command: 3, data: Some(vec!{0x00, 0xeb})};
    let encoded = wake::encode_packet(wp);

    //print_packet("Encoded packet:\t", &encoded);

    let decoded = wake::decode_packet(&encoded);
    match decoded {
        Ok(w) => {
            print!("\nDecoded packet:\t\tcmd  =  {:02X} ", w.0);
            print_packet("\t\t\tdata = ", &w.1);
        }
        Err(err) => println!("Error: {:?}", err),
    }

    // let wp = WakePacket{addr: Some(0x12), command: 3, data: Some(vec!{0x00, 0xeb})};
    // print!("\n{}", wp);
    // let wp = WakePacket{addr: Some(2), command: 4, data: None};
    // print!("\n{}", wp);
    // let mut a = String::from("DATA: ");
    // for x in v {
    //     a += &format!(" {:02X}", x);
    //     //print!("\n{:02X}", x);
    // }
    // print!("\n{}_", a);
    print!("_");
}
