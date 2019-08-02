use std::collections::HashMap;

use crate::tokenizer::Token;


pub struct Functions(HashMap<String, Vec<Token>>);

impl Functions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn define(&mut self, name: String, body: Vec<Token>) {
        self.0.insert(name, body);
    }

    pub fn get(&self, name: &str) -> Option<&Vec<Token>> {
        self.0.get(name)
    }
}
