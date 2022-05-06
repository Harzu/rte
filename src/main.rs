#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::cast_possible_truncation,
    // clippy::implicit_return,
    // clippy::shadow_reuse,
    // clippy::print_stdout,
    // clippy::wildcard_enum_match_arm,
    // clippy::else_if_without_else
)]

mod editor;
mod terminal;
mod document;
mod event_loop;

use std::error::Error;
use clap::{Command, Arg};
use editor::Editor;
use terminal::Terminal;
use document::Document;
use event_loop::EventLoop;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const FILE_PATH_ARG: &str = "filepath";

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let matches = Command::new(APP_NAME)
        .version(VERSION)
        .arg(
            Arg::new(FILE_PATH_ARG)
                .required(true)
                .index(1)
        )
        .get_matches();

    if let Some(filepath) = matches.value_of(FILE_PATH_ARG) {
        let document = Document::new(filepath)?;
        let terminal = Terminal::new()?;
        let editor = Editor::new(document, terminal)?;
        let mut event_loop = EventLoop::new(editor);
        event_loop.run()?;
    }

    Ok(())
}
