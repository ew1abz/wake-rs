extern crate serialport;
extern crate wake;

/// Main function doc string
fn main() {
    let ports = serialport::available_ports();
    for p in ports.iter(){
        println!("{:?}", p);
    }

    let xs: [u8; 5] = [1, 2, 3, 4, 5];
    let crc = wake::do_crc_array(&xs);
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

    destuffing(&mut v);
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
