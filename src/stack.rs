use std::fmt;

use crate::tokenizer::Token;


pub struct Stack(Vec<Value>);

impl Stack {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Result<Value, Error> {
        match self.0.pop() {
            Some(value) => Ok(value),
            None        => Err(Error::StackEmpty),
        }
    }
}


pub enum Value {
    Quote(Quote),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Quote(quote) => {
                write!(f, "[ ")?;
                for value in quote {
                    write!(f, "{} ", value)?;
                }
                write!(f, "]")?;
            }
            Value::String(string) => {
                write!(f, "{}", string)?;
            }
        }

        Ok(())
    }
}


pub type Quote = Vec<Token>;


pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Value,
    },
    StackEmpty,
}
