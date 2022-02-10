use std::io::{self, Stdout, Write};
use termion::{
    event::Key,
    input::TermRead,
    raw::{RawTerminal, IntoRawMode}
};

const EXIT_CHARACTER: char = 'q';

pub struct Editor {
    exit: bool,
    stdout: RawTerminal<Stdout>,
}

impl Editor {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Editor {
            exit: false,
            stdout: io::stdout().into_raw_mode()?,
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        while !self.exit {
            println!("please input key!\r");
            self.process_key()?;
        }

        println!("goodbye!\r");
        self.stdout.flush()
    }

    fn process_key(&mut self) -> Result<(), io::Error> {
        match self.next_key()? {
            Key::Ctrl(EXIT_CHARACTER) => { self.exit = true; },
            Key::Char(c) => { println!("your input: {}\r", c); },
            _ => ()
        }

        Ok(())
    }

    fn next_key(&self) -> Result<Key, io::Error> {
        match io::stdin().keys().next() {
            Some(key) => key,
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid input"
            ))
        }
    }
}