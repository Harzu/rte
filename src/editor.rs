use std::{
    fmt,
    io::{self, Stdout, Write},
};
use termion::{
    color,
    event::Key,
    input::TermRead,
    raw::{RawTerminal, IntoRawMode}
};

use crate::document::Document;

const EXIT_CHARACTER: char = 'q';
const PADDING_BUTTOM: u16 = 2;
const INFO_MESSAGE: &str = "CTRL-Q = exit";
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);

struct ScreenSize {
    width: u16,
    height: u16,
}

#[derive(Default)]
struct Position {
    x: u16,
    y: u16,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}|{})", self.x, self.y)
    }
}

pub struct Editor {
    exit: bool,
    stdout: RawTerminal<Stdout>,
    document: Document,
    screen_size: ScreenSize,
    cursor_position: Position,
}

impl Editor {
    pub fn new(document: Document) -> Result<Self, io::Error> {
        let (width, height) = termion::terminal_size()?;
        
        Ok(Editor {
            exit: false,
            stdout: io::stdout().into_raw_mode()?,
            document,
            screen_size: ScreenSize { width, height: height.saturating_sub(PADDING_BUTTOM) },
            cursor_position: Position::default(),
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
        print!("{}", termion::cursor::Goto::default());

        self.render_rows();
        self.render_status_bar();

        print!("{}", termion::cursor::Goto(
            self.cursor_position.x.saturating_add(1),
            self.cursor_position.y.saturating_add(1),
        ));

        self.stdout.flush()
    }

    fn render_rows(&self) {
        for row_num in 0..self.screen_size.height {
            print!("{}", termion::clear::CurrentLine);
            if let Some(row) = self.document.rows.get(row_num as usize) {
                println!("{}\r", row);
            } else {
                println!("\r");
            }
        }
    }

    fn render_status_bar(&self) {
        print!("{}", termion::clear::CurrentLine);

        let status_message = format!("cursor {}", self.cursor_position);
        let end_spaces = " ".repeat(
            self.screen_size.width.saturating_sub(status_message.len() as u16) as usize
        );
        let status = format!("{}{}", status_message, end_spaces);

        print!("{}{}", color::Bg(STATUS_BG_COLOR), color::Fg(STATUS_FG_COLOR));
        println!("{}\r", status);
        print!("{}{}", color::Bg(color::Reset), color::Fg(color::Reset));

        print!("{}", termion::clear::CurrentLine);
        print!("{}\r", String::from(INFO_MESSAGE));
    }

    fn process_key(&mut self) -> Result<(), io::Error> {
        match self.next_key()? {
            Key::Ctrl(EXIT_CHARACTER) => { self.exit = true; },
            Key::Char(c) => { println!("your input: {}\r", c); },
            Key::Up => {
                self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
            },
            Key::Down => {
                if self.cursor_position.y < self.screen_size.height - 1 {
                    self.cursor_position.y = self.cursor_position.y.saturating_add(1);
                }
            },
            Key::Left => {
                self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
            },
            Key::Right => {
                if self.cursor_position.x < self.screen_size.width - 1 {
                    self.cursor_position.x = self.cursor_position.x.saturating_add(1);
                }
            },
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