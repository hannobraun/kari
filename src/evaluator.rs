use std::io;

use crate::{
    builtins::Builtins,
    evaluate::{
        Error,
        Evaluate,
    },
    functions::{
        Functions,
    },
    parser::{
        self,
        Expression,
        Parser,
    },
    stack::Stack,
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

impl Evaluate for Evaluator {
    fn evaluate(&mut self, expressions: &mut Iterator<Item=Expression>)
        -> Result<(), Error>
    {
        for expression in expressions {
            match expression {
                Expression::Word(word) => {
                    if let Some(mut builtin) = self.builtins.take(&word) {
                        builtin
                            .input()
                            .take(&mut self.stack)?;
                        builtin.run(self)?;
                        builtin
                            .output()
                            .place(&mut self.stack);
                        for (name, body) in builtin.defines() {
                            self.functions.define(name, body);
                        }
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
