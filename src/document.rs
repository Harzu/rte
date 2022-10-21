use std::{
    fs::File,
    path::Path,
    io::{prelude::*, BufReader}
};

pub struct Document {
    pub rows: Vec<String>,
}

impl Document {
    pub fn new(file_path: &str) -> Result<Self, std::io::Error> {
        let mut document_rows = vec![];
        match File::open(Path::new(file_path)) {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    document_rows.push(line?);
                }
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    document_rows.push(String::new());
                },
                _ => return Err(e)
            },
        };

        Ok(Self{
            rows: document_rows,
        })
    }
}
