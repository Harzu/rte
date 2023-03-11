use crossbeam::channel::{select, unbounded, Receiver, RecvError};
#[cfg(not(feature = "extended-siginfo"))]
use signal_hook::consts::signal::SIGWINCH;
use signal_hook::iterator::{Handle, Signals};
use std::error;
use std::io::{self, Write};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use termion::color;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

const PADDING_BUTTON: u16 = 2;
const EXIT_CHARACTER: char = 'q';
const SAVE_CHARACTER: char = 's';

pub struct Terminal {
    stdout: AlternateScreen<RawTerminal<io::Stdout>>,
    size: ScreenSize,
    input_event_handler: InputEventHandler,
    syscall_signal_handler: SyscallHandler,
}

pub struct ScreenSize {
    width: u16,
    height: u16,
}

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Input(InputEvent),
    Syscall(SyscallEvent),
    Empty,
}

#[derive(Debug, Clone)]
pub enum SyscallEvent {
    WindowSizeChanged,
    Unsupported,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(KeyEvent),
    Empty,
    Unsupported,
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Backspace,
    Exit,
    SaveDocument,
    Unsupported,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.syscall_signal_handler.signals_handle.close();
        self.flush().unwrap();

        let input_handler_join_result = self
            .input_event_handler
            .join_handle
            .take()
            .expect("join handler is not found")
            .join()
            .expect("join thread operation is failed");

        if let Err(err) = input_handler_join_result {
            log::error!("{}", err);
        }

        let syscall_handler_join_result = self
            .syscall_signal_handler
            .join_handle
            .take()
            .expect("join handler is not found")
            .join()
            .expect("join thread operation is failed");

        if let Err(err) = syscall_handler_join_result {
            log::error!("{}", err);
        }
    }
}

impl Terminal {
    pub fn new() -> Result<Self, io::Error> {
        let raw_stdout = io::stdout().into_raw_mode()?;
        let mut terminal = Terminal {
            stdout: AlternateScreen::from(raw_stdout),
            size: ScreenSize {
                width: u16::MIN,
                height: u16::MIN,
            },
            input_event_handler: InputEventHandler::new(),
            syscall_signal_handler: SyscallHandler::new()?,
        };
        terminal.resize()?;
        Ok(terminal)
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }

    pub fn resize(&mut self) -> Result<(), io::Error> {
        let (width, height) = termion::terminal_size()?;
        self.size = ScreenSize {
            width,
            height: height.saturating_sub(PADDING_BUTTON),
        };
        Ok(())
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
        print!(
            "{}",
            termion::cursor::Goto(x.saturating_add(1), y.saturating_add(1),)
        );
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn set_row_color(background_rgb_color: (u8, u8, u8), foreground_rgb_color: (u8, u8, u8)) {
        print!(
            "{}{}",
            color::Bg(color::Rgb(
                background_rgb_color.0,
                background_rgb_color.1,
                background_rgb_color.2
            )),
            color::Fg(color::Rgb(
                foreground_rgb_color.0,
                foreground_rgb_color.1,
                foreground_rgb_color.2
            ))
        );
    }

    pub fn reset_line_color() {
        print!("{}{}", color::Bg(color::Reset), color::Fg(color::Reset));
    }

    pub fn pull_event(&self) -> Result<TerminalEvent, RecvError> {
        select! {
            recv(self.syscall_signal_handler.syscall_event_receiver) -> event => {
                Ok(TerminalEvent::Syscall(event?))
            },
            recv(self.input_event_handler.input_event_receiver) -> event => {
                Ok(TerminalEvent::Input(event?))
            }
            default(Duration::from_secs(1)) => {
                if let Some(input_event_handle) = &self.input_event_handler.join_handle {
                    if input_event_handle.is_finished() {
                        return Ok(TerminalEvent::Input(InputEvent::Key(KeyEvent::Exit)));
                    }
                }

                if let Some(syscall_event_handle) = &self.syscall_signal_handler.join_handle {
                    if syscall_event_handle.is_finished() {
                        return Ok(TerminalEvent::Input(InputEvent::Key(KeyEvent::Exit)));
                    }
                }

                Ok(TerminalEvent::Empty)
            }
        }
    }
}

struct SyscallHandler {
    join_handle: Option<JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>>>,
    syscall_event_receiver: Receiver<SyscallEvent>,
    signals_handle: Handle,
}

impl SyscallHandler {
    fn new() -> Result<Self, io::Error> {
        let (syscall_event_sender, syscall_event_receiver) = unbounded::<SyscallEvent>();
        let mut signals = Signals::new([SIGWINCH])?;
        let signals_handle = signals.handle();

        let join_handle =
            thread::spawn(move || -> Result<(), Box<dyn error::Error + Send + Sync>> {
                for signal in signals.forever() {
                    let signal_event = match signal {
                        SIGWINCH => SyscallEvent::WindowSizeChanged,
                        _ => SyscallEvent::Unsupported,
                    };
                    syscall_event_sender.send(signal_event)?;
                }
                Ok(())
            });

        Ok(SyscallHandler {
            join_handle: Some(join_handle),
            syscall_event_receiver,
            signals_handle,
        })
    }
}

struct InputEventHandler {
    join_handle: Option<JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>>>,
    input_event_receiver: Receiver<InputEvent>,
}

impl InputEventHandler {
    fn new() -> Self {
        let (input_event_sender, input_event_receiver) = unbounded::<InputEvent>();
        let join_handle =
            thread::spawn(move || -> Result<(), Box<dyn error::Error + Send + Sync>> {
                loop {
                    let input_event = InputEventHandler::next_key()?;
                    input_event_sender.send(input_event.clone())?;
                    if let InputEvent::Key(KeyEvent::Exit) = input_event {
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

    fn next_key() -> Result<InputEvent, io::Error> {
        if let Some(event) = io::stdin().events().next() {
            return match event? {
                Event::Key(key_event) => match key_event {
                    Key::Char(c) => Ok(InputEvent::Key(KeyEvent::Char(c))),
                    Key::Up => Ok(InputEvent::Key(KeyEvent::Up)),
                    Key::Down => Ok(InputEvent::Key(KeyEvent::Down)),
                    Key::Left => Ok(InputEvent::Key(KeyEvent::Left)),
                    Key::Right => Ok(InputEvent::Key(KeyEvent::Right)),
                    Key::Backspace => Ok(InputEvent::Key(KeyEvent::Backspace)),
                    Key::Ctrl(EXIT_CHARACTER) => Ok(InputEvent::Key(KeyEvent::Exit)),
                    Key::Ctrl(SAVE_CHARACTER) => Ok(InputEvent::Key(KeyEvent::SaveDocument)),
                    _ => Ok(InputEvent::Key(KeyEvent::Unsupported)),
                },
                _ => Ok(InputEvent::Unsupported),
            };
        }

        Ok(InputEvent::Empty)
    }
}
