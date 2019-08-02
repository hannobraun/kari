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
        let mut functions = HashMap::new();

        macro_rules! insert {
            ($name:expr, $builtin:expr) => {
                functions.insert(
                    String::from($name),
                    Function::Builtin(&$builtin)
                );
            }
        }

        insert!("print",  print);
        insert!("define", define);

        Self(functions)
    }

    pub fn define(&mut self, name: String, body: Quote) {
        self.0.insert(name, Function::Quote(body));
    }

    pub fn get(&self, name: &str) -> Option<Function> {
        self.0
            .get(name)
            .cloned()
    }
}


#[derive(Clone)]
pub enum Function {
    Builtin(&'static Builtin),
    Quote(Quote),
}


pub type Builtin = Fn(&mut Stack, &mut Functions) -> Result<(), stack::Error>;


pub fn print(stack: &mut Stack, _: &mut Functions) -> Result<(), stack::Error> {
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
