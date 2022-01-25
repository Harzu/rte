use std::io::{self, Read};
use termion::raw::IntoRawMode;

fn main() {
    let _raw_terminal = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();
    let bytes_iter = stdin.bytes();
    for byte in bytes_iter {
        let byte = byte.unwrap();
        let c = byte as char;
        if c == 'q' {
            break;
        }
        println!("{} {}\r", c, byte);
    }
}
