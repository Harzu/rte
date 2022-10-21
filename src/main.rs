mod editor;
mod document;

use clap::{Command, Arg};

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
    
    let file_path = matches.get_one::<String>(FILE_PATH_ARG).unwrap();
    let document = document::Document::new(file_path).unwrap();
    let mut editor = editor::Editor::new(document).unwrap();
    editor.run().unwrap();
}
