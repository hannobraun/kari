use std::{
    fmt,
    io,
};

use crate::{
    data::{
        expression::{
            self,
            Expression,
            List,
            Word,
        },
        span::{
            Span,
            WithSpan,
        },
        stack::{
            self,
            Stack,
        },
    },
    pipeline::parser,
};


pub trait Context {
    fn stack(&mut self) -> &mut Stack;
    fn define(&mut self, name: Word, body: List);
    fn load(&mut self, name: expression::String)
        -> Result<WithSpan<List>, Error>;
    fn evaluate(&mut self,
        operator:    Option<Span>,
        expressions: &mut Iterator<Item=Expression>,
    )
        -> Result<(), Error>;
}


#[derive(Debug)]
pub enum Error {
    Failure { operator: Span },
    UnknownFunction { name: String, span: Span },
    Expression(expression::Error),
    Io(io::Error),
    Parser(parser::Error),
    Stack(stack::Error),
}

impl Error {
    pub fn span(self) -> Option<Span> {
        match self {
            Error::Failure { operator }         => Some(operator),
            Error::UnknownFunction { span, .. } => Some(span),
            Error::Expression(error)            => error.span(),
            Error::Io(_)                        => None,
            Error::Parser(error)                => error.span(),
            Error::Stack(error)                 => error.span(),
        }
    }
}

impl From<expression::Error> for Error {
    fn from(from: expression::Error) -> Self {
        Error::Expression(from)
    }
}

impl From<io::Error> for Error {
    fn from(from: io::Error) -> Self {
        Error::Io(from)
    }
}

impl From<parser::Error> for Error {
    fn from(from: parser::Error) -> Self {
        Error::Parser(from)
    }
}

impl From<stack::Error> for Error {
    fn from(from: stack::Error) -> Self {
        Error::Stack(from)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Failure { .. } => {
                write!(f, "Explicit failure")
            }
            Error::UnknownFunction { name, .. } => {
                write!(f, "Unknown function: `{}`", name)
            }
            Error::Expression(error) => {
                error.fmt(f)
            }
            Error::Io(error) => {
                write!(f, "Error loading stream: {}", error)
            }
            Error::Parser(error) => {
                error.fmt(f)
            }
            Error::Stack(error) => {
                error.fmt(f)
            }
        }
    }
}
