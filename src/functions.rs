use std::collections::HashMap;

use crate::tokenizer::Token;


pub struct Functions(HashMap<String, Function>);

impl Functions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn define(&mut self, name: String, body: Quote) {
        self.0.insert(name, Function::Quote(body));
    }

    pub fn get(&self, name: &str) -> Option<&Function> {
        self.0.get(name)
    }
}


pub enum Function {
    Quote(Quote),
}


pub type Quote = Vec<Token>;
