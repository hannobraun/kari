use std::fmt;

use crate::tokenizer::Token;


pub type Stack = Vec<Value>;


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
