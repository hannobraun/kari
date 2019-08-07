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
    builtins:    Builtins,
    stack:       Stack,
    functions:   Functions,
    stack_trace: Vec<Span>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            builtins:    Builtins::new(),
            stack:       Stack::new(),
            functions:   Functions::new(),
            stack_trace: Vec::new(),
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
                    return Err(
                        Error {
                            kind:        error.into(),
                            stack_trace: self.stack_trace,
                        }
                    );
                }
            };

            let result = self.evaluate(
                None,
                &mut Some(expression).into_iter(),
            );
            if let Err(error) = result {
                return Err(
                    Error {
                        kind:        error.into(),
                        stack_trace: self.stack_trace,
                    }
                );
            }
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

    fn evaluate(&mut self,
        operator:    Option<Span>,
        expressions: &mut Iterator<Item=Expression>,
    )
        -> Result<(), context::Error>
    {
        if let Some(operator) = operator {
            self.stack_trace.push(operator);
        }

        for expression in expressions {
            if let expression::Kind::Word(word) = expression.kind {
                if let Some(list) = self.functions.get(&word) {
                    let list = list.clone();
                    self.evaluate(
                        Some(expression.span),
                        &mut list.0.into_iter(),
                    )?;
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

        if let Some(_) = operator {
            self.stack_trace.pop();
        }

        Ok(())
    }
}


pub struct Error {
    pub kind:        ErrorKind,
    pub stack_trace: Vec<Span>,
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match &self.kind {
            ErrorKind::Context(error) => error.span(),
            ErrorKind::Parser(error)  => error.span(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Context(error) => error.fmt(f),
            ErrorKind::Parser(error)  => error.fmt(f),
        }
    }
}


pub enum ErrorKind {
    Context(context::Error),
    Parser(parser::Error),
}

impl From<context::Error> for ErrorKind {
    fn from(from: context::Error) -> Self {
        ErrorKind::Context(from)
    }
}

impl From<parser::Error> for ErrorKind {
    fn from(from: parser::Error) -> Self {
        ErrorKind::Parser(from)
    }
}
