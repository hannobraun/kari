use std::collections::HashMap;

use crate::tokenizer::Token;


pub struct Functions(HashMap<String, Quote>);

impl Functions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn define(&mut self, name: String, body: Quote) {
        self.0.insert(name, body);
    }

    pub fn get(&self, name: &str) -> Option<&Quote> {
        self.0.get(name)
    }
}


pub type Quote = Vec<Token>;
