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
const DEFAULT_X_POSITION: usize = usize::MIN;
const DEFAULT_Y_POSITION: usize = usize::MIN;
struct ScreenSize {
    width: u16,
    height: u16,
}

#[derive(Default)]
struct Position {
    x: usize,
    y: usize,
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
    screen_offset: Position,
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
            screen_offset: Position::default(),
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        while !self.exit {
            self.change_offsets();
            self.render()?;
            self.process_key()?;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<(), io::Error> {
        print!("{}", termion::cursor::Hide);
        print!("{}", termion::cursor::Goto::default());

        self.render_rows();
        self.render_status_bar();

        print!("{}", termion::cursor::Goto(
            self.cursor_position.x.saturating_sub(self.screen_offset.x).saturating_add(1) as u16,
            self.cursor_position.y.saturating_sub(self.screen_offset.y).saturating_add(1) as u16,
        ));
        print!("{}", termion::cursor::Show);

        self.stdout.flush()
    }

    fn render_rows(&self) {
        for row_num in 0..self.screen_size.height {
            print!("{}", termion::clear::CurrentLine);
            if let Some(row) = self.document.rows.get(
                self.screen_offset.y.saturating_add(row_num as usize)
            ) {
                self.render_row(row);
            } else {
                println!("\r");
            }
        }
    }

    fn render_row(&self, row: &str) {
        let start = self.screen_offset.x;
        let end = start.saturating_add(self.screen_size.width as usize);

        let render_target: String = row
            .chars()
            .skip(start)
            .take(end - start)
            .collect();

        println!("{}\r", render_target);
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
            Key::Up => self.move_up(),
            Key::Down => self.move_down(),
            Key::Left => self.move_left(),
            Key::Right => self.move_right(),
            _ => ()
        }

        Ok(())
    }

    fn move_up(&mut self) {
        self.cursor_position.y = self.cursor_position.y.saturating_sub(1);

        let row_len = self.document.rows[self.cursor_position.y].len();
        if self.cursor_position.x > row_len {
            self.cursor_position.x = row_len;
        }
    }

    fn move_down(&mut self) {
        if self.cursor_position.y < self.document.rows.len() - 1 {
            self.cursor_position.y = self.cursor_position.y.saturating_add(1);

            let row_len = self.document.rows[self.cursor_position.y].len();
            if self.cursor_position.x > row_len {
                self.cursor_position.x = row_len;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_position.x == DEFAULT_X_POSITION && self.cursor_position.y != DEFAULT_Y_POSITION {
            self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
            self.cursor_position.x = self.document.rows[self.cursor_position.y].len();
        } else {
            self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
        }
    }

    fn move_right(&mut self) {
        if self.cursor_position.x < self.document.rows[self.cursor_position.y].len() {
            self.cursor_position.x = self.cursor_position.x.saturating_add(1);
        } else if self.cursor_position.y < self.document.rows.len() - 1 {
            self.cursor_position.y = self.cursor_position.y.saturating_add(1);
            self.cursor_position.x = DEFAULT_X_POSITION;
        }
    }

    pub fn change_offsets(&mut self) {
        let height = self.screen_size.height as usize;
        if self.cursor_position.y < self.screen_offset.y {
            self.screen_offset.y = self.cursor_position.y;
        } else if self.cursor_position.y >= self.screen_offset.y.saturating_add(height) {
            self.screen_offset.y = self.cursor_position.y
                .saturating_sub(height)    
                .saturating_add(1);
        }

        let width = self.screen_size.width as usize;
        if self.cursor_position.x < self.screen_offset.x {
            self.screen_offset.x = self.cursor_position.x;
        } else if self.cursor_position.x >= self.screen_offset.x.saturating_add(width) {
            self.screen_offset.x = self.cursor_position.x
                .saturating_sub(width)
                .saturating_add(1);
        }
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