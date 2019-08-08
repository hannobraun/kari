use std::{
    collections::HashMap,
    fmt,
};

use crate::{
    builtins::{
        Builtins,
        context::{
            self,
            Context,
        },
    },
    data::{
        expression::{
            self,
            Expression,
            List,
        },
        span::Span,
        stack::Stack,
    },
    pipeline::{
        self,
        parser,
    },
};


pub struct Evaluator {
    builtins:    Builtins,
    stack:       Stack,
    functions:   HashMap<String, List>,
    stack_trace: Vec<Span>,
}

impl Evaluator {
    pub fn run<Parser>(parser: Parser)
        -> Result<(), Error>
        where Parser: pipeline::Stage<Item=Expression, Error=parser::Error>
    {
        let evaluator = Self {
            builtins:    Builtins::new(),
            stack:       Stack::new(),
            functions:   HashMap::new(),
            stack_trace: Vec::new(),
        };

        evaluator.evaluate_expressions(parser)
    }

    fn evaluate_expressions<Parser>(mut self, mut parser: Parser)
        -> Result<(), Error>
        where Parser: pipeline::Stage<Item=Expression, Error=parser::Error>
    {
        loop {
            let expression = match parser.next() {
                Ok(expression) => {
                    expression
                }
                Err(parser::Error::EndOfStream) => {
                    return Ok(());
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
    }
}

impl Context for Evaluator {
    fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    fn define(&mut self, name: String, body: List) {
        self.functions.insert(name, body);
    }

    fn evaluate(&mut self,
        operator:    Option<Span>,
        expressions: &mut Iterator<Item=Expression>,
    )
        -> Result<(), context::Error>
    {
        let mut pop_operator = false;
        if let Some(operator) = operator {
            self.stack_trace.push(operator);
            pop_operator = true;
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
                    builtin(expression.span, self)?;
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

        if pop_operator {
            self.stack_trace.pop();
        }

        Ok(())
    }
}


pub struct Error {
    pub kind:        ErrorKind,
    pub stack_trace: Vec<Span>,
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

impl ErrorKind {
    pub fn span(self) -> Option<Span> {
        match self {
            ErrorKind::Context(error) => error.span(),
            ErrorKind::Parser(error)  => error.span(),
        }
    }
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
