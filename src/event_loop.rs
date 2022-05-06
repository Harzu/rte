use std::{
    thread,
    error::Error,
    time::Duration,
};
use termion::event::Key;
use crossbeam::channel::{
    unbounded,
    select,
    tick,
    Receiver, Sender
};
use crate::{Terminal, Editor};

pub struct EventLoop {
    editor: Editor
}

struct Signal;

impl EventLoop {
    pub fn new(editor: Editor) -> Self {
        EventLoop {
            editor
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let (
            close_tx, close_rx,
        ): (Sender<Signal>, Receiver<Signal>) = unbounded();
        let (
            key_getter_close_tx, key_getter_close_rx,
        ): (Sender<Signal>, Receiver<Signal>) = (close_tx.clone(), close_rx.clone());
        let (
            render_tx, render_rx,
        ): (Sender<Signal>, Receiver<Signal>) = unbounded();
        let (
            next_key_signal_tx, next_key_signal_rx,
        ): (Sender<Signal>, Receiver<Signal>) = unbounded();
        let (
            key_tx, key_rx,
        ): (Sender<Key>, Receiver<Key>) = unbounded();
        
        let key_getter_thread = thread::spawn(
            move || {
                loop {
                    select! {
                        recv(key_getter_close_rx) -> _ => break,
                        recv(next_key_signal_rx) -> _ => {
                            let key = Terminal::get_key()?;
                            key_tx.send(key)?;
                        }
                    }
                }
    
                Ok(())
            }
        );

        let ticker = tick(Duration::from_millis(100));

        render_tx.send(Signal)?;
        next_key_signal_tx.send(Signal)?;

        loop {
            select! {
                recv(close_rx) -> _ => break,
                recv(render_rx) -> _ => {
                    self.editor.change_offsets();
                    self.editor.render()?;
                },
                recv(key_rx) -> key => {
                    self.editor.process_key(
                        key?,
                        || {
                            key_getter_close_tx.send(Signal).unwrap();
                            close_tx.send(Signal).unwrap();
                        },
                    )?;
                    render_tx.send(Signal)?;
                    next_key_signal_tx.send(Signal)?;
                },
                recv(ticker) -> _ => {
                    self.editor.resize()?;
                    render_tx.send(Signal)?;
                },
            };            
        }

        key_getter_thread.join().unwrap()
    }
}
