extern crate serialport;
extern crate wake;

use wake::*;

/// Main function doc string
fn main() {
    let ports = serialport::available_ports();
    for p in ports.iter(){
        println!("{:?}", p);
    }
}
