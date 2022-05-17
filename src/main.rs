#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::cast_possible_truncation,
    clippy::single_match,
)]

mod editor;
mod search;
mod terminal;
mod document;
mod constants;
mod event_loop;

use clap::{Command, Arg};
use editor::Editor;
use terminal::Terminal;
use document::Document;
use event_loop::EventLoop;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const FILE_PATH_ARG: &str = "filepath";

fn main() {
    let matches = Command::new(APP_NAME)
        .version(VERSION)
        .arg(
            Arg::new(FILE_PATH_ARG)
                .required(true)
                .index(1)
        )
        .get_matches();

    if let Some(filepath) = matches.value_of(FILE_PATH_ARG) {
        let document = Document::new(filepath).unwrap();
        let terminal = Terminal::new().unwrap();
        let editor = Editor::new(document, terminal).unwrap();
        
        EventLoop::new(editor)
            .run()
            .unwrap();
    }
}
