use crate::document::Document;
use crate::terminal::{InputEvent, KeyEvent, SyscallEvent, Terminal, TerminalEvent};
use std::error;
use std::fmt;
use std::io;

const INFO_MESSAGE: &str = "CTRL-Q = exit | CTRL-S = save";
const STATUS_BG_COLOR: (u8, u8, u8) = (239, 239, 239);
const STATUS_FG_COLOR: (u8, u8, u8) = (63, 63, 63);
const DEFAULT_X_POSITION: usize = usize::MIN;
const DEFAULT_Y_POSITION: usize = usize::MIN;

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
    terminal: Terminal,
    document: Document,
    cursor_position: Position,
    screen_offset: Position,
}

impl Editor {
    pub fn new(terminal: Terminal, document: Document) -> Self {
        Editor {
            exit: bool::default(),
            terminal,
            document,
            cursor_position: Position::default(),
            screen_offset: Position::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.render()?;
        while !self.exit {
            self.process_event()?;
            self.render()?;
        }
        Ok(())
    }

    fn render(&mut self) -> Result<(), io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_to_default_position();

        self.change_offsets();
        self.render_rows();
        self.render_status_bar();

        Terminal::cursor_to_position(
            self.cursor_position.x.saturating_sub(self.screen_offset.x) as u16,
            self.cursor_position.y.saturating_sub(self.screen_offset.y) as u16,
        );
        Terminal::cursor_show();

        self.terminal.flush()
    }

    fn render_rows(&self) {
        for row_num in 0..self.terminal.height() {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .try_get_row(self.screen_offset.y.saturating_add(row_num as usize))
            {
                self.render_row(row);
            } else {
                println!("\r");
            }
        }
    }

    fn render_row(&self, row: &str) {
        let start = self.screen_offset.x;
        let end = start.saturating_add(self.terminal.width() as usize);
        let render_target: String = row.chars().skip(start).take(end - start).collect();
        println!("{render_target}\r");
    }

    fn render_status_bar(&self) {
        Terminal::clear_current_line();

        let mut document_is_modified_flag = "";
        if self.document.is_modified() {
            document_is_modified_flag = "[+] ";
        }

        let status_message = format!(
            "{}{} {}",
            document_is_modified_flag, self.document.file_path, self.cursor_position
        );
        let end_spaces = " ".repeat(
            self.terminal
                .width()
                .saturating_sub(status_message.len() as u16) as usize,
        );
        let status = format!("{status_message}{end_spaces}");

        Terminal::set_row_color(STATUS_BG_COLOR, STATUS_FG_COLOR);
        println!("{status}\r");
        Terminal::reset_line_color();

        Terminal::clear_current_line();
        print!("{}\r", String::from(INFO_MESSAGE));
    }

    fn process_event(&mut self) -> Result<(), Box<dyn error::Error>> {
        match self.terminal.pull_event()? {
            TerminalEvent::Input(input_event) => self.process_input_event(input_event)?,
            TerminalEvent::Syscall(syscall_event) => self.process_syscall_event(&syscall_event)?,
            TerminalEvent::Empty => (),
        }
        Ok(())
    }

    fn process_syscall_event(&mut self, event: &SyscallEvent) -> Result<(), io::Error> {
        match event {
            SyscallEvent::WindowSizeChanged => self.terminal.resize()?,
            SyscallEvent::Unsupported => (),
        }
        Ok(())
    }

    fn process_input_event(&mut self, event: InputEvent) -> Result<(), io::Error> {
        if let InputEvent::Key(key_event) = event {
            match key_event {
                KeyEvent::Char(c) => self.add_char(c),
                KeyEvent::Exit => {
                    self.exit = true;
                },
                KeyEvent::SaveDocument => self.document.save()?,
                KeyEvent::Backspace => self.remove_char(),
                KeyEvent::Up => self.move_up(),
                KeyEvent::Down => self.move_down(),
                KeyEvent::Left => self.move_left(),
                KeyEvent::Right => self.move_right(),
                KeyEvent::Unsupported => (),
            }
        }
        Ok(())
    }

    fn add_char(&mut self, c: char) {
        self.document
            .insert_char(self.cursor_position.y, self.cursor_position.x, c);
        self.move_right();
    }

    fn remove_char(&mut self) {
        if self.cursor_position.x > DEFAULT_X_POSITION {
            let prev_index = self.cursor_position.x.saturating_sub(1);
            self.document
                .remove_char(self.cursor_position.y, prev_index);
            self.move_left();
        } else if self.cursor_position.y > DEFAULT_Y_POSITION {
            let current_row_num = self.cursor_position.y;
            self.move_left();
            self.document.join_row_with_previous(current_row_num);
        }
    }

    fn move_up(&mut self) {
        self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
        let row_len = self.document.get_row(self.cursor_position.y).len();
        if self.cursor_position.x > row_len {
            self.cursor_position.x = row_len;
        }
    }

    fn move_down(&mut self) {
        if self.cursor_position.y < self.document.len() - 1 {
            self.cursor_position.y = self.cursor_position.y.saturating_add(1);
            let row_len = self.document.get_row(self.cursor_position.y).len();
            if self.cursor_position.x > row_len {
                self.cursor_position.x = row_len;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_position.x == DEFAULT_X_POSITION
            && self.cursor_position.y != DEFAULT_Y_POSITION
        {
            self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
            self.cursor_position.x = self.document.get_row(self.cursor_position.y).len();
        } else {
            self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
        }
    }

    fn move_right(&mut self) {
        if self.cursor_position.x < self.document.get_row(self.cursor_position.y).len() {
            self.cursor_position.x = self.cursor_position.x.saturating_add(1);
        } else if self.cursor_position.y < self.document.len() - 1 {
            self.cursor_position.y = self.cursor_position.y.saturating_add(1);
            self.cursor_position.x = DEFAULT_X_POSITION;
        }
    }

    pub fn change_offsets(&mut self) {
        let height = self.terminal.height() as usize;
        if self.cursor_position.y < self.screen_offset.y {
            self.screen_offset.y = self.cursor_position.y;
        } else if self.cursor_position.y >= self.screen_offset.y.saturating_add(height) {
            self.screen_offset.y = self
                .cursor_position
                .y
                .saturating_sub(height)
                .saturating_add(1);
        }

        let width = self.terminal.width() as usize;
        if self.cursor_position.x < self.screen_offset.x {
            self.screen_offset.x = self.cursor_position.x;
        } else if self.cursor_position.x >= self.screen_offset.x.saturating_add(width) {
            self.screen_offset.x = self
                .cursor_position
                .x
                .saturating_sub(width)
                .saturating_add(1);
        }
    }
}
