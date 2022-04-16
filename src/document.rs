use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, BufReader, LineWriter};

use crate::editor::NEW_LINE_CHARACTER;

pub struct Document {
    pub rows: Vec<String>,
    _file_path: String,
}

impl Document {
    pub fn new(file_path: &str) -> Result<Self, std::io::Error> {
        let mut document_rows = vec![];

        let path = Path::new(file_path);
        match File::open(&path) {
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {},
                _ => return Err(e)
            },
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    document_rows.push(line?);
                }
            },
        };

        if document_rows.len() == 0 {
            document_rows.push(String::new());
        }

        Ok(Self{
            rows: document_rows,
            _file_path: file_path.to_string(),
        })
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let file = File::create(&self._file_path.to_string())?;
        let mut file = LineWriter::new(file);

        for (index, row) in self.rows.iter().enumerate() {
            let mut buf: Vec<u8> = vec![];
            for byte in row.bytes() {
                buf.push(byte);
            }
            if index != self.rows.len() - 1 {
                buf.push(NEW_LINE_CHARACTER as u8);
            }
            file.write_all(&buf[..])?;
        }

        file.flush()?;
        Ok(())
    }
}