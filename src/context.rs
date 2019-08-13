use std::{
    fmt,
    io,
};

use crate::{
    data::{
        expr,
        span::Span,
        stack::{
            self,
            Stack,
        },
    },
    pipeline::parser,
};


pub trait Context {
    fn stack(&mut self) -> &mut Stack;
    fn output(&mut self) -> &mut io::Write;
    fn define(&mut self, name: expr::Symbol, body: expr::List);
    fn load(&mut self, name: expr::String)
        -> Result<expr::List, Error>;
    fn evaluate(&mut self,
        operator:    Option<Span>,
        expressions: &mut Iterator<Item=expr::Any>,
    )
        -> Result<(), Error>;
}


#[derive(Debug)]
pub enum Error {
    Failure { operator: Span },
    UnknownFunction { name: String, span: Span },
    Expr(expr::Error),
    Io(io::Error),
    Parser(parser::Error),
    Stack(stack::Error),
}

impl Error {
    pub fn spans(self, spans: &mut Vec<Span>) {
        match self {
            Error::Failure { operator }         => spans.push(operator),
            Error::UnknownFunction { span, .. } => spans.push(span),

            Error::Expr(error)   => error.spans(spans),
            Error::Parser(error) => error.spans(spans),
            Error::Stack(error)  => error.spans(spans),

            Error::Io(_) => (),
        }
    }
}

impl From<expr::Error> for Error {
    fn from(from: expr::Error) -> Self {
        Error::Expr(from)
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
            Error::Expr(error) => {
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
