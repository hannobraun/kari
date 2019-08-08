use std::{
    borrow::Cow,
    collections::HashMap,
    io,
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
    interpreter::{
        error::Error,
        stream::Stream,
    },
    pipeline::{
        self,
        parser,
    },
};


pub struct Evaluator {
    streams:     HashMap<String, Box<Stream>>,
    builtins:    Builtins,
    stack:       Stack,
    functions:   HashMap<String, List>,
    stack_trace: Vec<Span>,
}

impl Evaluator {
    pub fn run(name: Cow<str>, mut stream: Box<Stream>) -> bool {
        let mut evaluator = Self {
            streams:     HashMap::new(),
            builtins:    Builtins::new(),
            stack:       Stack::new(),
            functions:   HashMap::new(),
            stack_trace: Vec::new(),
        };

        let pipeline = pipeline::new(
            name.clone().into_owned(),
            &mut stream,
        );

        let result = evaluator.evaluate_expressions(pipeline);
        if let Err(error) = result {
            evaluator.streams.insert(
                name.into_owned(),
                stream,
            );

            if let Err(error) = error.print(&mut evaluator.streams) {
                print!("Error printing error: {}\n", error)
            }
            return false;
        }

        true
    }

    fn evaluate_expressions<Parser>(&mut self, mut parser: Parser)
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
                            stack_trace: self.stack_trace.clone(),
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
                        stack_trace: self.stack_trace.clone(),
                    }
                );
            }
        }
    }
}

impl Context for Evaluator
    where Stream: io::Read + io::Seek
{
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
