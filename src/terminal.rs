use std::io::{self, Write};
use termion::{
    color,
    event::Key,
    input::TermRead,
    screen::AlternateScreen,
    raw::{IntoRawMode, RawTerminal},
};

const PADDING_BUTTON: u16 = 2;

pub struct Terminal {
    stdout: AlternateScreen<RawTerminal<io::Stdout>>,
    size: ScreenSize,
}

pub struct ScreenSize {
    width: u16,
    height: u16,
}

pub enum KeyEvent {
    Char(char),
    Ctrl(char),
    Up,
    Down,
    Left,
    Right,
    Backspace,
    Unknown,
}

impl Terminal {
    pub fn new() -> Result<Self, io::Error> {
        let (width, height) = termion::terminal_size()?;
        let raw_stdout = io::stdout().into_raw_mode()?;

        Ok(Terminal {
            stdout: AlternateScreen::from(raw_stdout),
            size: ScreenSize {
                width,
                height: height.saturating_sub(PADDING_BUTTON),
            }
        })
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }

    pub fn width(&self) -> u16 {
        self.size.width
    }

    pub fn height(&self) -> u16 {
        self.size.height
    }

    pub fn next_key() -> Result<KeyEvent, io::Error> {
        if let Some(key) = io::stdin().keys().next() {
            return match key? {
                Key::Char(c) => Ok(KeyEvent::Char(c)),
                Key::Ctrl(c) => Ok(KeyEvent::Ctrl(c)),
                Key::Up => Ok(KeyEvent::Up),
                Key::Down => Ok(KeyEvent::Down),
                Key::Left => Ok(KeyEvent::Left),
                Key::Right => Ok(KeyEvent::Right),
                Key::Backspace => Ok(KeyEvent::Backspace),
                _ => Ok(KeyEvent::Unknown),
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "invalid input"))
    }

    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }

    pub fn cursor_to_default_position() {
        print!("{}", termion::cursor::Goto::default());
    }

    pub fn cursor_to_position(x: u16, y: u16) {
        print!("{}", termion::cursor::Goto(
            x.saturating_add(1),
            y.saturating_add(1),
        ));
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn set_row_color(background_rgb_color: (u8, u8, u8), foreground_rgb_color: (u8, u8, u8)) {
        print!(
            "{}{}",
            color::Bg(color::Rgb(background_rgb_color.0, background_rgb_color.1, background_rgb_color.2)),
            color::Fg(color::Rgb(foreground_rgb_color.0, foreground_rgb_color.1, foreground_rgb_color.2))
        );
    }

    pub fn reset_line_color() {
        print!("{}{}", color::Bg(color::Reset), color::Fg(color::Reset));
    }
}
