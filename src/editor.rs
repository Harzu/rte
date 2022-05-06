use std::{fmt, io, cmp, error};
use termion::{color, event::Key};
use crate::{Terminal, Document};

pub const NEW_LINE_CHARACTER: char = '\n';

const EXIT_CHARACTER: char = 'q';
const SAVE_CHARACTER: char = 's';
const PADDING_BUTTOM: u16 = 2;
const INFO_MESSAGE: &str = "CTRL-Q = exit | CTRL-S = save";
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
    terminal: Terminal,
    document: Document,
    screen_size: ScreenSize,
    cursor_position: Position,
    screen_offset: Position,
}

impl Editor {
    pub fn new(document: Document, terminal: Terminal) -> Result<Self, io::Error> {
        let (width, height) = termion::terminal_size()?;
        
        Ok(Editor {
            terminal,
            document,
            screen_size: ScreenSize { width, height: height.saturating_sub(PADDING_BUTTOM) },
            cursor_position: Position::default(),
            screen_offset: Position::default(),
        })
    }

    pub fn resize(&mut self) -> Result<(), io::Error> {
        let (width, height) = termion::terminal_size()?;
        self.screen_size = ScreenSize { width, height: height.saturating_sub(PADDING_BUTTOM) };
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), io::Error> {
        print!("{}", termion::cursor::Hide);
        print!("{}", termion::cursor::Goto::default());

        self.render_rows();
        self.render_status_bar();

        print!("{}", termion::cursor::Goto(
            self.cursor_position.x.saturating_sub(self.screen_offset.x).saturating_add(1) as u16,
            self.cursor_position.y.saturating_sub(self.screen_offset.y).saturating_add(1) as u16,
        ));
        print!("{}", termion::cursor::Show);
        
        self.terminal.flush()
    }

    fn render_rows(&self) {
        for row_num in 0..self.screen_size.height {
            print!("{}", termion::clear::CurrentLine);
            if let Some(row) = self.document.rows
                .get(self.screen_offset.y
                .saturating_add(row_num  as usize))
            {
                self.render_row(row);
            } else {
                println!("\r");
            }
        }
    }

    fn render_row(&self, row: &str) {
        let mut start = self.screen_offset.x;
        let mut end = start.saturating_add(self.screen_size.width as usize);

        end = cmp::min(end, row.len());
        start = cmp::min(start, end);

        let render: String = row
            .chars()
            .into_iter()
            .skip(start as usize)
            .take((end - start) as usize)
            .collect();

        println!("{}\r", render);
    }

    fn render_status_bar(&self) {
        print!("{}", termion::clear::CurrentLine);

        let status_message = format!(
            "cursor {} | offset {}",
            self.cursor_position,
            self.screen_offset,
        );
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

    pub fn process_key<F: FnOnce()>(
        &mut self,
        key: Key,
        exit_func: F
    ) -> Result<(), Box<dyn error::Error + Send + Sync>>
    {
        match key {
            Key::Ctrl(EXIT_CHARACTER) => exit_func(),
            Key::Ctrl(SAVE_CHARACTER) => self.document.save()?,
            Key::Char(c) => self.add_char(c),
            Key::Backspace => self.remove_char(),
            Key::Up => self.move_up(),
            Key::Down => self.move_down(),
            Key::Left => self.move_left(),
            Key::Right => self.move_right(),
            _ => ()
        }

        Ok(())
    }

    fn add_char(&mut self, c: char) {
        if let Some(row) = self.document.rows.get_mut(self.cursor_position.y) {
            if c == NEW_LINE_CHARACTER {
                let new_row = row.split_off(self.cursor_position.x);
                self.document.rows.insert(self.cursor_position.y.saturating_add(1), new_row);
            } else {
                row.insert(self.cursor_position.x, c);
            }

            self.move_right();
        }
    }

    fn remove_char(&mut self) {
        if self.cursor_position.x > DEFAULT_X_POSITION {
            if let Some(row) = self.document.rows.get_mut(self.cursor_position.y) {
                row.remove(self.cursor_position.x.saturating_sub(1));
                self.move_left();
            }
        } else if self.cursor_position.y > DEFAULT_Y_POSITION {
            let curr_index = self.cursor_position.y;
            let prev_index = self.cursor_position.y.saturating_sub(1);
            let curr_row = self.document.rows.get(curr_index);
            let prev_row = self.document.rows.get(prev_index);

            if prev_row.is_none() {
                return;
            }

            let mut new_row = String::from(prev_row.unwrap());
            if let Some(row) = curr_row {
                new_row.push_str(row);
            }

            self.move_left();
            self.document.rows[prev_index] = new_row;
            self.document.rows.remove(curr_index);
        }
    }

    fn move_up(&mut self) {
        self.cursor_position.y = self.cursor_position.y.saturating_sub(1);

        if let Some(row) = self.document.rows.get(self.cursor_position.y) {
            if self.cursor_position.x > row.len() {
                self.cursor_position.x = row.len();
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_position.y < self.document.rows.len() - 1 {
            self.cursor_position.y = self.cursor_position.y.saturating_add(1);

            if let Some(row) = self.document.rows.get(self.cursor_position.y) {
                if self.cursor_position.x > row.len() {
                    self.cursor_position.x = row.len();
                }
            }
        }
    }

    fn move_left(&mut self) {        
        if self.cursor_position.x == DEFAULT_X_POSITION && self.cursor_position.y != DEFAULT_Y_POSITION {
            self.move_up();
            if let Some(row) = self.document.rows.get(self.cursor_position.y) {
                self.cursor_position.x = row.len();
            }
        } else {
            self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
        }
    }

    fn move_right(&mut self) {
        if let Some(row) = self.document.rows.get(self.cursor_position.y) {
            if self.cursor_position.x < row.len() {
                self.cursor_position.x = self.cursor_position.x.saturating_add(1);
            } else if self.cursor_position.y < self.document.rows.len() - 1 {
                self.move_down();
                self.cursor_position.x = DEFAULT_X_POSITION;
            }
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
}