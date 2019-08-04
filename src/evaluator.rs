use std::io;

use crate::{
    builtins::Builtins,
    functions::{
        Functions,
    },
    parser::{
        self,
        Expression,
        List,
        Parser,
    },
    stack::{
        self,
        Stack,
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

    pub fn run<R>(&mut self, mut parser: Parser<R>)
        -> Result<(), Error>
        where R: io::Read
    {
        loop {
            let expression = match parser.next() {
                Ok(expression)                  => expression,
                Err(parser::Error::EndOfStream) => break,
                Err(error)                      => return Err(error.into()),
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
        -> Result<(), Error>
    {
        for expression in expressions {
            match expression {
                Expression::Word(word) => {
                    if let Some(mut builtin) = self.builtins.take(&word) {
                        builtin.run(self)?;
                        self.builtins.put_back(builtin);
                        continue;
                    }
                    if let Some(list) = self.functions.get(&word) {
                        let list = list.clone();
                        self.evaluate(&mut list.into_iter())?;
                        continue;
                    }

                    return Err(Error::UnknownFunction(
                        word.to_string())
                    );
                }
                expression => {
                    self.stack.push(expression);
                }
            }
        }

        Ok(())
    }
}


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

