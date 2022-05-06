use std::io::{self, Stdout, Write};
use termion::{
    event::Key,
    input::TermRead,
    screen::AlternateScreen,
    raw::{RawTerminal, IntoRawMode},
};

pub struct Terminal {
     stdout: AlternateScreen<RawTerminal<Stdout>>,
}

impl Terminal {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Terminal {
            stdout: AlternateScreen::from(io::stdout().into_raw_mode()?),
        })
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }

    pub fn get_key() -> Result<Key, io::Error> {
        match io::stdin().keys().next() {
            Some(key) => key,
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid input"
            ))
        }
    }
}