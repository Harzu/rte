use std::{
    fs::File,
    path::Path,
    io::{prelude::*, BufReader, LineWriter}
};
use crate::constants::NEW_LINE_CHARACTER;

pub struct Document {
    pub rows: Vec<String>,
    pub file_path: String,
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

        if document_rows.is_empty(){
            document_rows.push(String::new());
        }

        Ok(Self{
            rows: document_rows,
            file_path: file_path.to_string(),
        })
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let file = File::create(&self.file_path)?;
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