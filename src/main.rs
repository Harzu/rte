use std::io::{self, Read};

fn main() {
    let stdin = io::stdin();
    let bytes_iter = stdin.bytes();
    for byte in bytes_iter {
        let byte = byte.unwrap();
        println!("{} {}", byte as char, byte);
    }
}
