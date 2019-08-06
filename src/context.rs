use std::fmt;

use crate::{
    parser::{
        self,
        Expression,
        List,
    },
    stack::{
        self,
        Stack,
    },
};


pub trait Context {
    fn stack(&mut self) -> &mut Stack;
    fn define(&mut self, name: String, body: List);
    fn evaluate(&mut self, expressions: &mut Iterator<Item=Expression>)
        -> Result<(), Error>;
}


#[derive(Debug)]
pub enum Error {
    Parser(parser::Error),
    UnknownFunction(String),
    Stack(stack::Error),
}

impl From<stack::Error> for Error {
    fn from(from: stack::Error) -> Self {
        Error::Stack(from)
    }
}

impl From<parser::Error> for Error {
    fn from(from: parser::Error) -> Self {
        Error::Parser(from)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Parser(error) => {
                write!(f, "Parser error:\n{:?}", error)?;
            }
            Error::UnknownFunction(name) => {
                write!(f, "Unknown function: {}", name)?;
            }
            Error::Stack(error) => {
                write!(f, "Stack error:\n{:?}", error)?;
            }
        }

        Ok(())
    }
}
