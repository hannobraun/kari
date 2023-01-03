use std::{fmt, io};

use termion::{color, style};

use crate::{
    call_stack::CallStack,
    functions::{self, Function, Functions, Scope, Signatures},
    pipeline::{parser, tokenizer::Span},
    stack::{self, Stack},
    value::{self, cast::TypeError, v},
};

pub trait Context<Host> {
    fn functions(&mut self) -> &mut Functions<Function<Host>>;

    fn stack(&mut self) -> &mut Stack;

    fn call_stack(&mut self) -> &mut CallStack;

    fn output(&mut self) -> &mut dyn io::Write;

    fn load(&mut self, name: v::String, scope: Scope)
        -> Result<v::List, Error>;

    fn evaluate_value(
        &mut self,
        host: &mut Host,
        scope: Scope,
        value: value::Any,
    ) -> Result<(), Error>;

    fn evaluate_list(
        &mut self,
        host: &mut Host,
        list: v::List,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error {
    Caller,
    DefineFunction(functions::DefineError),
    Failure,
    FunctionNotFound {
        name: String,
        stack: Stack,
        candidates: Signatures,
        scope: String,
    },
    ModuleNotFound(String),
    Io(io::Error),
    Parser(parser::Error),
    Stack(stack::Error),
    Type(TypeError),
}

impl Error {
    pub fn sources<'r>(&'r self, sources: &mut Vec<&'r Span>) {
        match self {
            Error::Caller => (),
            Error::DefineFunction(_) => (),
            Error::Failure => (),
            Error::FunctionNotFound { .. } => (),
            Error::ModuleNotFound(_) => (),

            Error::Parser(error) => error.sources(sources),
            Error::Stack(error) => error.sources(sources),
            Error::Type(error) => error.sources(sources),

            Error::Io(_) => (),
        }
    }

    pub fn write_hint(&self, stderr: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Error::FunctionNotFound {
                stack,
                candidates,
                scope,
                ..
            } => {
                if !candidates.is_empty() {
                    writeln!(
                        stderr,
                        "{}Values on stack:{}",
                        color::Fg(color::Cyan),
                        color::Fg(color::Reset),
                    )?;
                    write!(
                        stderr,
                        "    {}{}{}{}{}\n\n",
                        style::Bold,
                        color::Fg(color::LightWhite),
                        stack,
                        color::Fg(color::Reset),
                        style::Reset,
                    )?;

                    writeln!(
                        stderr,
                        "{}Candidate functions:{}",
                        color::Fg(color::Cyan),
                        color::Fg(color::Reset),
                    )?;
                    for candidate in candidates {
                        writeln!(stderr, "    {:?}", candidate)?;
                    }
                } else {
                    writeln!(
                        stderr,
                        "{}No functions of that name found.{}",
                        color::Fg(color::Cyan),
                        color::Fg(color::Reset),
                    )?;
                }

                writeln!(
                    stderr,
                    "{}Scope: {}{}{}`{}`{}{}",
                    color::Fg(color::Cyan),
                    color::Fg(color::Reset),
                    style::Bold,
                    color::Fg(color::LightWhite),
                    scope,
                    color::Fg(color::Reset),
                    style::Reset,
                )?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl From<stack::Error> for Error {
    fn from(from: stack::Error) -> Self {
        Error::Stack(from)
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Caller => {
                write!(f, "No caller found")
            }
            Error::Failure { .. } => {
                write!(f, "Explicit failure")
            }
            Error::FunctionNotFound { name, .. } => {
                write!(f, "No matching function found: `{}`", name)
            }
            Error::ModuleNotFound(name) => {
                write!(f, "Module not found: {}", name)
            }
            Error::Io(error) => {
                write!(f, "Error loading stream: {}", error)
            }

            Error::DefineFunction(error) => error.fmt(f),
            Error::Parser(error) => error.fmt(f),
            Error::Stack(error) => error.fmt(f),
            Error::Type(error) => error.fmt(f),
        }
    }
}
