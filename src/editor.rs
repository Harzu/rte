use std::io::{self, Stdout, Write};
use termion::{
    event::Key,
    input::TermRead,
    raw::{RawTerminal, IntoRawMode}
};

const EXIT_CHARACTER: char = 'q';

struct ScreenSize {
    width: u16,
    height: u16,
}

pub struct Editor {
    exit: bool,
    stdout: RawTerminal<Stdout>,
    screen_size: ScreenSize,
}

impl Editor {
    pub fn new() -> Result<Self, io::Error> {
        let (width, height) = termion::terminal_size()?;
        
        Ok(Editor {
            exit: false,
            stdout: io::stdout().into_raw_mode()?,
            screen_size: ScreenSize { width, height },
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        while !self.exit {
            self.render()?;
            self.process_key()?;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<(), io::Error> {
        for row_num in 0..self.screen_size.height {
            if row_num == self.screen_size.height / 2 {
                let message = "Hello from rust-text-editor";
                let padding = " ".repeat(
                    (self.screen_size.width / 2 + 1) as usize - message.len() / 2
                );

                println!("~{}{}\r", padding, message);
            } else {
                println!("~\r");
            }
        }

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