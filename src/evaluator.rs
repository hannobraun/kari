use std::io;

use crate::{
    builtins::Builtins,
    context::{
        Context,
        Error,
    },
    functions::{
        Functions,
    },
    parser::{
        self,
        Expression,
        List,
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
                    if let Some(list) = self.functions.get(&word) {
                        let list = list.clone();
                        self.evaluate(&mut list.into_iter())?;
                        continue;
                    }
                    if let Some(builtin) = self.builtins.builtin(&word) {
                        builtin.run(self)?;
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
