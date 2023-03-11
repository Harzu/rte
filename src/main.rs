#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::cast_possible_truncation)]

mod document;
mod editor;
mod terminal;

use clap::{Arg, Command};
use log::{debug, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::str::FromStr;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const EDIT_FILE_PATH_ARG: &str = "edit_file_path";
const LOG_FILE_PATH_ARG: &str = "log_file_path";
const LOG_LEVEL_ARG: &str = "log_level";
const LOG_FILE_APPENDER_KEY: &str = "log_file";

fn main() {
    let matches = Command::new(APP_NAME)
        .version(VERSION)
        .arg(
            Arg::new(LOG_FILE_PATH_ARG)
                .required(false)
                .long("log-file")
                .short('f')
                .default_value("/tmp/rte.log"),
        )
        .arg(
            Arg::new(LOG_LEVEL_ARG)
                .required(false)
                .long("log-level")
                .short('l')
                .default_value("info"),
        )
        .arg(Arg::new(EDIT_FILE_PATH_ARG).required(true).index(1))
        .get_matches();

    let log_file_path = matches.get_one::<String>(LOG_FILE_PATH_ARG).unwrap();
    let log_level = matches.get_one::<String>(LOG_LEVEL_ARG).unwrap();
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} | {l} - {m}{n}")))
        .build(log_file_path)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build(LOG_FILE_APPENDER_KEY, Box::new(file_appender)))
        .build(
            Root::builder()
                .appender(LOG_FILE_APPENDER_KEY)
                .build(LevelFilter::from_str(log_level).unwrap()),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    let edit_file_path = matches.get_one::<String>(EDIT_FILE_PATH_ARG).unwrap();
    let document = document::Document::new(edit_file_path).unwrap();
    let terminal = terminal::Terminal::new().unwrap();
    let mut editor = editor::Editor::new(terminal, document);

    debug!("RTE open {} file", edit_file_path);
    editor.run().unwrap();
}
