mod editor;
mod document;

use clap::{Command, Arg};

use editor::Editor;
use document::Document;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const FILE_PATH_ARG: &str = "filepath";

fn main() -> Result<(), std::io::Error> {
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
        let mut editor = Editor::new(document)?;
        editor.run()?;
    }

    Ok(())
}
