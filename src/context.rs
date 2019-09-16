use std::{
    fmt,
    io,
};

use termion::{
    color,
    style,
};

use crate::{
    data::{
        functions::{
            self,
            Functions,
        },
        span::Span,
        stack::{
            self,
            Stack,
        },
        types::{
            Type,
            TypeError,
        },
        value,
    },
    function::Function,
    pipeline::parser,
};


pub trait Context<H> {
    fn stack(&mut self) -> &mut Stack;

    fn output(&mut self) -> &mut dyn io::Write;

    fn load(&mut self, name: value::String)
        -> Result<value::List, Error>;

    fn evaluate_value(&mut self,
        functions: &mut Functions<Function<H>>,
        operator:  Option<Span>,
        expr:      value::Any,
    )
        -> Result<(), Error>;

    fn evaluate_list(&mut self,
        functions: &mut Functions<Function<H>>,
        operator:  Option<Span>,
        list:      value::List,
    )
        -> Result<(), Error>;
}


#[derive(Debug)]
pub enum Error {
    DefineFunction(functions::DefineError),
    Failure { operator: Span },
    FunctionNotFound {
        name:       String,
        span:       Span,
        stack:      Stack,
        candidates: Vec<Vec<&'static dyn Type>>,
    },
    Io(io::Error),
    Parser(parser::Error),
    Stack(stack::Error),
    Type(TypeError),
}

impl Error {
    pub fn spans<'r>(&'r self, spans: &mut Vec<&'r Span>) {
        match self {
            Error::DefineFunction(_)             => (),
            Error::Failure { operator }          => spans.push(operator),
            Error::FunctionNotFound { span, .. } => spans.push(span),

            Error::Parser(error) => error.spans(spans),
            Error::Stack(error)  => error.spans(spans),
            Error::Type(error)   => error.spans(spans),

            Error::Io(_)    => (),
        }
    }

    pub fn write_hint(&self, stderr: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Error::FunctionNotFound { stack, candidates, .. } => {
                if candidates.len() > 0 {
                    write!(
                        stderr,
                        "{}Values on stack:{}\n",
                        color::Fg(color::Cyan),
                        color::Fg(color::Reset),
                    )?;
                    write!(
                        stderr,
                        "    {}{}{}{}{}\n\n",
                        style::Bold, color::Fg(color::LightWhite),
                        stack,
                        color::Fg(color::Reset), style::Reset,
                    )?;

                    write!(
                        stderr,
                        "{}Candidate functions:{}\n",
                        color::Fg(color::Cyan),
                        color::Fg(color::Reset),
                    )?;
                    for candidate in candidates {
                        write!(stderr, "    {:?}\n", candidate)?;
                    }
                }
                else {
                    write!(
                        stderr,
                        "{}No functions of that name found in scope.{}\n",
                        color::Fg(color::Cyan),
                        color::Fg(color::Reset),
                    )?;
                }

                Ok(())
            },
            _ => {
                Ok(())
            }
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

impl From<functions::DefineError> for Error {
    fn from(from: functions::DefineError) -> Self {
        Error::DefineFunction(from)
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
            Error::DefineFunction(error) => {
                error.fmt(f)
            }
            Error::Failure { .. } => {
                write!(f, "Explicit failure")
            }
            Error::FunctionNotFound { name, .. } => {
                write!(f, "No matching function found: `{}`", name)
            }
            Error::Io(error) => {
                write!(f, "Error loading stream: {}", error)
            }

            Error::Parser(error) => error.fmt(f),
            Error::Stack(error)  => error.fmt(f),
            Error::Type(error)   => error.fmt(f),
        }
    }
}
