use std::{
    thread, io,
    error::Error,
    time::Duration,
};
use libc::c_int;
use termion::event::Key;
use crossbeam::channel::{
    unbounded, select,
    Receiver, Sender,
};
#[cfg(not(feature = "extended-siginfo"))]
use signal_hook::{
    consts::signal::SIGWINCH,
    iterator::Signals,
};
use crate::{Terminal, Editor};

pub struct EventLoop {
    editor: Editor
}

struct Signal;

type SignalChan = (Sender<Signal>, Receiver<Signal>);
type KeyChan = (Sender<Key>, Receiver<Key>);
type IOErrorChan = (Sender<io::Error>, Receiver<io::Error>);

impl EventLoop {
    pub fn new(editor: Editor) -> Self {
        EventLoop {
            editor
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        const SIGNALS: &[c_int] = &[SIGWINCH];
        let mut sigs = Signals::new(SIGNALS)?;

        let (close_tx, close_rx): SignalChan = unbounded();
        let (render_tx, render_rx): SignalChan = unbounded();
        let (next_key_signal_tx, next_key_signal_rx): SignalChan = unbounded();
        let (key_tx, key_rx): KeyChan = unbounded();
        let (io_error_tx, io_error_rx): IOErrorChan = unbounded();

        let key_getter_thread = thread::spawn(
            move || {
                loop {
                    select! {
                        recv(close_rx) -> _ => break,
                        recv(next_key_signal_rx) -> _ => {
                            match Terminal::get_key() {
                                Ok(key) => { key_tx.send(key).unwrap(); },
                                Err(err) => { io_error_tx.send(err).unwrap(); }, 
                            }
                        },
                    }
                }
            }
        );

        render_tx.send(Signal)?;
        next_key_signal_tx.send(Signal)?;
        
        loop {
            select! {
                recv(render_rx) -> _ => {
                    self.editor.change_offsets();
                    self.editor.render()?;
                },
                recv(key_rx) -> key => {
                    self.editor.process_key(key?)?;

                    if self.editor.is_close() {
                        close_tx.send(Signal)?;
                        break;
                    }
                    
                    render_tx.send(Signal)?;
                    next_key_signal_tx.send(Signal)?;
                },
                recv(io_error_rx) -> _ => {
                    close_tx.send(Signal)?;
                    break;
                }
                default(Duration::from_millis(100)) => {
                    if sigs.pending().next().is_some() {
                        self.editor.resize()?;
                        render_tx.send(Signal)?; 
                    }
                }
            };            
        }

        key_getter_thread
            .join()
            .unwrap();
        
        Ok(())
    }
}
