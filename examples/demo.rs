extern crate wake;

fn print_packet(header: &str, v: &Vec<u8>) {
    print!("\n{}\t", header);
    for x in v {
        print!("{:02X} ", x);
    }
}

/// Simple wake_rs API demo
fn main() {
    let wp = wake::Packet {
        address: Some(0x12),
        command: 3,
        data: Some(vec![0x00, 0xeb]),
    };
    let encoded = wake::encode_packet(wp);
    print_packet("Encoded packet:\t", &encoded);

    let decoded = wake::decode_packet(&encoded);
    println!("Decoded packet: {}", decoded.unwrap());
}
