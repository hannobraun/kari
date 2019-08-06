use crate::{
    builtins::Builtins,
    context::{
        Context,
        Error,
    },
    expression::{
        self,
        Expression,
        List,
    },
    functions::{
        Functions,
    },
    parser,
    stack::Stack,
    stream::Stream,
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
                    Error::UnknownFunction {
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
