use std::fmt;

use crate::{
    core::{
        builtins::Builtins,
        context::{
            self,
            Context,
        },
        expression::{
            self,
            Expression,
            List,
        },
        functions::{
            Functions,
        },
        span::Span,
        stack::Stack,
    },
    pipeline::{
        parser,
        stream::Stream,
    },
};


pub struct Evaluator {
    builtins:  Builtins,
    stack:     Stack,
    functions: Functions,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            builtins:  Builtins::new(),
            stack:     Stack::new(),
            functions: Functions::new(),
        }
    }

    pub fn run<Parser>(mut self, mut parser: Parser)
        -> Result<(), Error>
        where Parser: Stream<Item=Expression, Error=parser::Error>
    {
        loop {
            let expression = match parser.next() {
                Ok(expression) => {
                    expression
                }
                Err(parser::Error::EndOfStream) => {
                    break;
                }
                Err(error) => {
                    return Err(error.into());
                }
            };

            self.evaluate(&mut Some(expression).into_iter())?;
        }

        Ok(())
    }
}

impl Context for Evaluator {
    fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    fn define(&mut self, name: String, body: List) {
        self.functions.define(name, body);
    }

    fn evaluate(&mut self, expressions: &mut Iterator<Item=Expression>)
        -> Result<(), context::Error>
    {
        for expression in expressions {
            if let expression::Kind::Word(word) = expression.kind {
                if let Some(list) = self.functions.get(&word) {
                    let list = list.clone();
                    self.evaluate(&mut list.0.into_iter())?;
                    continue;
                }
                if let Some(builtin) = self.builtins.builtin(&word) {
                    builtin.run(expression.span, self)?;
                    continue;
                }

                return Err(
                    context::Error::UnknownFunction {
                        name: word.to_string(),
                        span: expression.span,
                    }
                );
            }
            else {
                self.stack.push::<Expression>(expression);
            }
        }

        Ok(())
    }
}


pub enum Error {
    Context(context::Error),
    Parser(parser::Error),
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::Context(error) => error.span(),
            Error::Parser(error)  => error.span(),
        }
    }
}

impl From<context::Error> for Error {
    fn from(from: context::Error) -> Self {
        Error::Context(from)
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
            Error::Context(error) => error.fmt(f),
            Error::Parser(error)  => error.fmt(f),
        }
    }
}
