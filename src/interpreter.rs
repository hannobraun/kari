use std::io;

use crate::{
    builtins::Builtins,
    functions::{
        Functions,
    },
    parser::{
        self,
        Expression,
        Parser,
    },
    stack::{
        self,
        Stack,
    },
};


pub struct Interpreter {
    builtins:  Builtins,
    stack:     Stack,
    functions: Functions,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
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

            self.evaluate(Some(expression))?;
        }

        Ok(())
    }

    fn evaluate<Expressions>(&mut self, expressions: Expressions)
        -> Result<(), Error>
        where Expressions: IntoIterator<Item=Expression>
    {
        for expression in expressions {
            match expression {
                Expression::Word(word) => {
                    match word.as_str() {
                        "run" => {
                            let arg = self.stack.pop()?;
                            match arg {
                                Expression::List(list) => {
                                    self.evaluate(list)?;
                                }
                                arg => {
                                    return Err(
                                        Error::Stack(
                                            stack::Error::TypeError {
                                                expected: "list",
                                                actual:   arg,
                                            }
                                        )
                                    );
                                }
                            };
                        }
                        word => {
                            if let Some(builtin) = self.builtins.get(word) {
                                builtin
                                    .input()
                                    .take(&mut self.stack)?;
                                builtin.run(
                                    &mut self.stack,
                                    &mut self.functions,
                                );
                                continue;
                            }
                            if let Some(list) = self.functions.get(word) {
                                let list = list.clone();
                                self.evaluate(list)?;
                                continue;
                            }

                            return Err(Error::UnknownFunction(
                                word.to_string())
                            );
                        }
                    }
                }
                expression => {
                    self.stack.push(expression);
                }
            }
        }

        Ok(())
    }
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
