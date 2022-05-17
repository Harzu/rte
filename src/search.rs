use crate::constants::NEW_LINE_CHARACTER;

#[derive(Default)]
pub struct SearchEngine {
    pub query: String
}

impl SearchEngine {
    pub fn add_char_to_query(&mut self, c: char) {
        if c != NEW_LINE_CHARACTER {
            self.query.push(c);
        }
    }

    pub fn remove_char_to_query(&mut self) {
        self.query.pop();
    }

    pub fn clear_query(&mut self) {
        self.query = String::new();
    }
}