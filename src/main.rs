use std::io::{self, Read};
use termion::raw::IntoRawMode;

const MASK: u8 = 0b0001_1111;

fn main() {
    let _raw_terminal = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();
    let bytes_iter = stdin.bytes();
    for byte in bytes_iter {
        let byte = byte.unwrap();
        println!("byte: {}\r", byte);
        println!("char: {}\r", byte as char);
        println!("binary: {:#b}\r", byte);
        if byte == ('q' as u8 & MASK) {
            break;
        }
    }
}
