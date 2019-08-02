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

    pub fn pop<T>(&mut self) -> Result<T, Error> where T: Type {
        match self.0.pop() {
            Some(value) => T::check(value),
            None        => Err(Error::StackEmpty),
        }
    }
}


pub enum Value {
    Number(u32),
    Quote(Quote),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(number) => number.fmt(f),
            Value::String(string) => string.fmt(f),

            Value::Quote(quote) => {
                write!(f, "[ ")?;
                for value in quote {
                    write!(f, "{} ", value)?;
                }
                write!(f, "]")
            }
        }
    }
}


pub type Quote = Vec<Token>;


pub trait Type : Sized {
    fn check(value: Value) -> Result<Self, Error>;
}

impl Type for Value {
    fn check(value: Value) -> Result<Self, Error> {
        Ok(value)
    }
}

impl Type for Quote {
    fn check(value: Value) -> Result<Self, Error> {
        match value {
            Value::Quote(quote) => {
                Ok(quote)
            }
            value => {
                Err(Error::TypeError {
                    expected: "quote",
                    actual:   value,
                })
            }
        }
    }
}


pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Value,
    },
    StackEmpty,
}
