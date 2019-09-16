use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io,
    rc::Rc,
};

use crate::{
    context::{
        self,
        Context,
    },
    data::{
        functions::{
            self,
            Functions,
        },
        span::Span,
        stack::Stack,
        value::{
            self,
            Value as _,
        },
    },
    function::{
        Function,
        Host,
    },
    interpreter::{
        error::Error,
        stream::Stream,
    },
    pipeline::{
        self,
        Stage as _,
        parser,
    },
};


pub struct Evaluator<H> {
    streams: HashMap<String, Box<dyn Stream>>,
    stdout:  Box<dyn io::Write>,
    stderr:  Box<dyn io::Write>,

    host:        Host<H>,
    stack:       Stack,
    stack_trace: Vec<Span>,
}

impl<H> Evaluator<H> {
    pub fn new(
        stdout: Box<dyn io::Write>,
        stderr: Box<dyn io::Write>,
        host:   H,
    )
        -> Self
    {
        Self {
            streams: HashMap::new(),
            stdout,
            stderr,

            host:        Rc::new(RefCell::new(host)),
            stack:       Stack::new(),
            stack_trace: Vec::new(),
        }
    }

    pub fn run(mut self,
            name:      Cow<str>,
        mut prelude:   Box<dyn Stream>,
        mut program:   Box<dyn Stream>,
        mut functions: Functions<Function<H>>,
    )
        -> bool
    {
        let prelude_name = "<prelude>";

        let prelude_pipeline = pipeline::new(
            prelude_name.into(),
            &mut prelude,
        );
        self.evaluate_expressions(prelude_pipeline, &mut functions)
            .expect("Error while evaluating prelude");

        // We panic on errors in the prelude itself, but errors in other modules
        // might still produce stack traces with spans that refer to the
        // prelude.
        self.streams.insert(
            prelude_name.into(),
            prelude,
        );

        let pipeline = pipeline::new(
            name.clone().into_owned(),
            &mut program,
        );

        let result = self.evaluate_expressions(pipeline, &mut functions);
        if let Err(error) = result {
            self.streams.insert(
                name.into_owned(),
                program,
            );

            if let Err(error) =
                error.print(&mut self.streams, &mut self.stderr)
            {
                print!("Error printing error: {}\n", error)
            }
            return false;
        }

        true
    }

    fn evaluate_expressions<Parser>(&mut self,
        mut parser:    Parser,
            functions: &mut Functions<Function<H>>,
    )
        -> Result<(), Error>
        where Parser: pipeline::Stage<Item=value::Any, Error=parser::Error>
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

            let result = self.evaluate_value(
                functions,
                None,
                expression,
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

impl<H> Context<H> for Evaluator<H> {
    fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    fn output(&mut self) -> &mut dyn io::Write {
        &mut self.stdout
    }

    fn load(&mut self, name: value::String)
        -> Result<value::List, context::Error>
    {
        let     path   = format!("kr/src/{}.kr", name.inner);
        let mut stream = File::open(&path)?;

        let mut parser = pipeline::new(path.clone(), &mut stream);
        let mut values = Vec::new();

        loop {
            match parser.next() {
                Ok(value) => {
                    values.push(value);
                }
                Err(parser::Error::EndOfStream) => {
                    break;
                }
                Err(error) => {
                    return Err(error.into());
                }
            }
        }

        self.streams.insert(path, Box::new(stream));

        let start = values
            .first()
            .map(|expression| expression.span.clone())
            .unwrap_or(Span::default());
        let end = values
            .last()
            .map(|expression| expression.span.clone())
            .unwrap_or(Span::default());

        Ok(value::List::new(values, start.merge(&end)))
    }

    fn evaluate_value(&mut self,
        functions: &mut Functions<Function<H>>,
        operator:  Option<Span>,
        expr:      value::Any,
    )
        -> Result<(), context::Error>
    {
        let mut pop_operator = false;
        if let Some(operator) = operator {
            self.stack_trace.push(operator);
            pop_operator = true;
        }

        if let value::Kind::Word(word) = expr.kind {
            match functions.get(functions.root_scope(), &word, &self.stack) {
                Ok(f) => {
                    match f {
                        Function::Builtin(f) => {
                            f(
                                self.host.clone(),
                                self,
                                functions,
                                expr.span,
                            )?;
                        }
                        Function::UserDefined { body } => {
                            self.evaluate_list(
                                functions,
                                Some(expr.span),
                                body,
                            )?;
                        }
                    }
                }
                Err(functions::GetError { candidates }) => {
                    return Err(
                        context::Error::FunctionNotFound {
                            name:  word,
                            span:  expr.span,
                            stack: self.stack.clone(),
                            candidates,
                        }
                    );
                }
            }
        }
        else {
            self.stack.push::<value::Any>(expr);
        }

        if pop_operator {
            self.stack_trace.pop();
        }

        Ok(())
    }

    fn evaluate_list(&mut self,
        functions: &mut Functions<Function<H>>,
        operator:  Option<Span>,
        list:      value::List,
    )
        -> Result<(), context::Error>
    {
        for expr in list {
            self.evaluate_value(functions, operator.clone(), expr)?;
        }

        Ok(())
    }
}
