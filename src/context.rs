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
        types::TypeError,
    },
    pipeline::parser,
    scope::{
        self,
        Function,
        Scope,
    },
};


pub trait Context<H> {
    fn stack(&mut self) -> &mut Stack;

    fn output(&mut self) -> &mut dyn io::Write;

    fn load(&mut self, name: expr::String)
        -> Result<expr::List, Error>;

    fn evaluate_expr(&mut self,
        scope:    &mut Scope<Function<H>>,
        operator: Option<Span>,
        expr:     expr::Any,
    )
        -> Result<(), Error>;

    fn evaluate_list(&mut self,
        scope:    &mut Scope<Function<H>>,
        operator: Option<Span>,
        list:     expr::List,
    )
        -> Result<(), Error>;
}


#[derive(Debug)]
pub enum Error {
    Failure { operator: Span },
    UnknownFunction { name: String, span: Span },
    Io(io::Error),
    Parser(parser::Error),
    Scope(scope::Error),
    Stack(stack::Error),
    Type(TypeError),
}

impl Error {
    pub fn spans(self, spans: &mut Vec<Span>) {
        match self {
            Error::Failure { operator }         => spans.push(operator),
            Error::UnknownFunction { span, .. } => spans.push(span),

            Error::Parser(error) => error.spans(spans),
            Error::Stack(error)  => error.spans(spans),
            Error::Type(error)   => error.spans(spans),

            Error::Scope(_) => (),
            Error::Io(_)    => (),
        }
    }
}

impl From<TypeError> for Error {
    fn from(from: TypeError) -> Self {
        Error::Type(from)
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

impl From<scope::Error> for Error {
    fn from(from: scope::Error) -> Self {
        Error::Scope(from)
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
            Error::Io(error) => {
                write!(f, "Error loading stream: {}", error)
            }

            Error::Scope(error)  => error.fmt(f),
            Error::Parser(error) => error.fmt(f),
            Error::Stack(error)  => error.fmt(f),
            Error::Type(error)   => error.fmt(f),
        }
    }
}
