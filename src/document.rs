use std::fs::File;
use std::io::{self, prelude::*, BufReader, LineWriter};
use std::path::Path;

const NEW_LINE_CHARACTER: char = '\n';

pub struct Document {
    rows: Vec<String>,
    pub file_path: String,
    is_modified: bool,
}

impl Document {
    pub fn new(file_path: &str) -> Result<Self, io::Error> {
        let mut document_rows = vec![];
        match File::open(Path::new(file_path)) {
            Ok(file) => {
                if file.metadata()?.len() > 0 {
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        document_rows.push(line?);
                    }
                } else {
                    document_rows.push(String::new());
                }
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    document_rows.push(String::new());
                },
                _ => return Err(e),
            },
        };

        Ok(Self {
            is_modified: false,
            rows: document_rows,
            file_path: String::from(file_path),
        })
    }

    pub fn save(&mut self) -> Result<(), io::Error> {
        let file = File::create(&self.file_path)?;
        let mut writer = LineWriter::new(file);

        for (row_num, row_content) in self.rows.iter().enumerate() {
            let mut buf: Vec<u8> = row_content.bytes().collect();
            if row_num != self.rows.len() - 1 {
                buf.push(NEW_LINE_CHARACTER as u8);
            }

            writer.write_all(&buf[..])?;
        }

        writer.flush()?;
        self.is_modified = false;
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn get_row(&self, row_num: usize) -> &String {
        &self.rows[row_num]
    }

    pub fn try_get_row(&self, row_num: usize) -> Option<&String> {
        self.rows.get(row_num)
    }

    pub fn insert_char(&mut self, row_num: usize, index: usize, c: char) {
        if c == NEW_LINE_CHARACTER {
            let new_row = self.rows[row_num].split_off(index);
            self.rows.insert(row_num.saturating_add(1), new_row);
        } else {
            self.rows[row_num].insert(index, c);
        }
        self.is_modified = true;
    }

    pub fn remove_char(&mut self, row_num: usize, index: usize) {
        self.rows[row_num].remove(index);
        self.is_modified = true;
    }

    pub fn join_row_with_previous(&mut self, row_num: usize) {
        let row = self.rows[row_num].clone();
        self.rows[row_num.saturating_sub(1)].push_str(&row);
        self.rows.remove(row_num);
        self.is_modified = true;
    }
}
