use std::collections::HashMap;

use crate::{
    stack::{
        self,
        Quote,
        Stack,
        Value,
    },
    tokenizer::Token,
};


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


#[derive(Clone)]
pub enum Function {
    Quote(Quote),
}


pub fn print(stack: &mut Stack) -> Result<(), stack::Error> {
    let arg = stack.pop().unwrap();
    print!("{}", arg);

    Ok(())
}

pub fn define(stack: &mut Stack, functions: &mut Functions)
    -> Result<(), stack::Error>
{
    let name = stack.pop().unwrap();
    let name = match name {
        Value::Quote(mut quote) => {
            assert_eq!(quote.len(), 1);
            quote.pop().unwrap()
        }
        arg => {
            return Err(stack::Error::TypeError {
                expected: "quote",
                actual:   arg,
            });
        }
    };
    let name = match name {
        Token::Word(word) => {
            word
        }
        token => {
            panic!(
                "Unexpected token: {}\n",
                token,
            );
        }
    };

    let body = stack.pop().unwrap();
    let body = match body {
        Value::Quote(quote) => {
            quote
        }
        arg => {
            return Err(stack::Error::TypeError {
                expected: "quote",
                actual:   arg,
            });
        }
    };

    functions.define(name, body);

    Ok(())
}
