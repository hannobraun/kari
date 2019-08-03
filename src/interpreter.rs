use std::io;

use crate::{
    functions::{
        Function,
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
    stack:     Stack,
    functions: Functions,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
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
                            match self.functions.get(word) {
                                Some(Function::Builtin(builtin)) => {
                                    builtin.run(
                                        &mut self.stack,
                                        &mut self.functions,
                                    )?;
                                }
                                Some(Function::List(list)) => {
                                    self.evaluate(list)?;
                                }
                                None => {
                                    return Err(Error::UnknownFunction(
                                        word.to_string())
                                    );
                                }
                            }
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
