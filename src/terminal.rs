use std::io::{self, Write};
use std::error;
use std::time::Duration;
use std::thread::{self, JoinHandle};
use crossbeam::channel::{unbounded, select, Receiver, RecvError};
use termion::{
    color,
    event::Key,
    input::TermRead,
    screen::AlternateScreen,
    raw::{IntoRawMode, RawTerminal},
};

const PADDING_BUTTON: u16 = 2;
const EXIT_CHARACTER: char = 'q';
const SAVE_CHARACTER: char = 's';

pub struct Terminal {
    stdout: AlternateScreen<RawTerminal<io::Stdout>>,
    size: ScreenSize,
    input_event_handler: InputEventHandler,
}

pub struct ScreenSize {
    width: u16,
    height: u16,
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Exit,
    SaveDocument,
    Backspace,
    Unsupported,
    Empty,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.input_event_handler
            .join_handle
            .take()
            .expect("join handler is not found")
            .join()
            .expect("join thread operation is failed")
            .unwrap();
    }
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
            },
            input_event_handler: InputEventHandler::new(),
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

    pub fn pull_key_event(&self) -> Result<KeyEvent, RecvError> {
        select! {
            recv(self.input_event_handler.input_event_receiver) -> event => {
                event
            }
            default(Duration::from_secs(1)) => {
                if let Some(input_event_handle) = &self.input_event_handler.join_handle {
                    if input_event_handle.is_finished() {
                        return Ok(KeyEvent::Exit);
                    }
                }

                Ok(KeyEvent::Empty)
            }
        }
    }
}

struct InputEventHandler {
    join_handle: Option<JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>>>,
    input_event_receiver: Receiver<KeyEvent>,
}

impl InputEventHandler {
    fn new() -> Self {
        let (input_event_sender, input_event_receiver) = unbounded::<KeyEvent>();
        let join_handle = thread::spawn(move || -> Result<(), Box<dyn error::Error + Send + Sync>> {
            loop {
                let input_event = InputEventHandler::next_key()?;
                input_event_sender.send(input_event.clone())?;
                if let KeyEvent::Exit = input_event {
                    break;
                }
            }
            Ok(())
        });

        InputEventHandler {
            join_handle: Some(join_handle),
            input_event_receiver,
        }
    }

    fn next_key() -> Result<KeyEvent, io::Error> {
        if let Some(key) = io::stdin().keys().next() {
            return match key? {
                Key::Char(c) => Ok(KeyEvent::Char(c)),
                Key::Up => Ok(KeyEvent::Up),
                Key::Down => Ok(KeyEvent::Down),
                Key::Left => Ok(KeyEvent::Left),
                Key::Right => Ok(KeyEvent::Right),
                Key::Backspace => Ok(KeyEvent::Backspace),
                Key::Ctrl(EXIT_CHARACTER) => Ok(KeyEvent::Exit),
                Key::Ctrl(SAVE_CHARACTER) => Ok(KeyEvent::SaveDocument),
                _ => Ok(KeyEvent::Unsupported)
            };
        }

        Ok(KeyEvent::Empty)
    }
}
