use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io,
    rc::Rc,
};

use crate::{
    builtins,
    context::{
        self,
        Context,
    },
    data::{
        expr::{
            self,
            Expr as _,
        },
        span::Span,
        stack::Stack,
    },
    functions::{
        Builtin,
        Extension,
        Functions,
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


pub struct Evaluator<Host> {
    streams: HashMap<String, Box<dyn Stream>>,
    stdout:  Box<dyn io::Write>,
    stderr:  Box<dyn io::Write>,

    host:        Rc<RefCell<Host>>,
    extensions:  Functions<Extension<Host>>,
    builtins:    Functions<Builtin>,
    stack:       Stack,
    functions:   Functions<expr::List>,
    stack_trace: Vec<Span>,
}

impl<Host> Evaluator<Host> {
    pub fn new(
        stdout:     Box<dyn io::Write>,
        stderr:     Box<dyn io::Write>,
        host:       Host,
        extensions: Functions<Extension<Host>>,
    )
        -> Self
    {
        let mut builtins = Functions::new();
        builtins::builtins(&mut builtins);

        Self {
            streams: HashMap::new(),
            stdout,
            stderr,

            host:        Rc::new(RefCell::new(host)),
            extensions,
            builtins,
            stack:       Stack::new(),
            functions:   Functions::new(),
            stack_trace: Vec::new(),
        }
    }

    pub fn run(&mut self,
            name:    Cow<str>,
        mut prelude: Box<dyn Stream>,
        mut program: Box<dyn Stream>,
    )
        -> bool
    {
        let prelude = pipeline::new(
            String::from("<prelude>"),
            &mut prelude,
        );
        self.evaluate_expressions(prelude)
            .expect("Error while evaluating prelude");

        let pipeline = pipeline::new(
            name.clone().into_owned(),
            &mut program,
        );

        let result = self.evaluate_expressions(pipeline);
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

    fn evaluate_expressions<Parser>(&mut self, mut parser: Parser)
        -> Result<(), Error>
        where Parser: pipeline::Stage<Item=expr::Any, Error=parser::Error>
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

            let result = self.evaluate_expr(
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

impl<Host> Context for Evaluator<Host> {
    fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    fn output(&mut self) -> &mut dyn io::Write {
        &mut self.stdout
    }

    fn define(&mut self, name: expr::Symbol, body: expr::List)
        -> Result<(), context::Error>
    {
        self.functions.define(name.inner, &[], body)?;
        Ok(())
    }

    fn load(&mut self, name: expr::String)
        -> Result<expr::List, context::Error>
    {
        let     path   = format!("kr/src/{}.kr", name.inner);
        let mut stream = File::open(&path)?;

        let mut parser      = pipeline::new(path.clone(), &mut stream);
        let mut expressions = Vec::new();
        loop {
            match parser.next() {
                Ok(expression)                  => expressions.push(expression),
                Err(parser::Error::EndOfStream) => break,
                Err(error)                      => return Err(error.into()),
            }
        }

        self.streams.insert(path, Box::new(stream));

        let start = expressions
            .first()
            .map(|expression| expression.span.clone())
            .unwrap_or(Span::default());
        let end = expressions
            .last()
            .map(|expression| expression.span.clone())
            .unwrap_or(Span::default());

        Ok(expr::List::new(expressions, start.merge(end)))
    }

    fn evaluate_expr(&mut self,
        operator:   Option<Span>,
        expression: expr::Any,
    )
        -> Result<(), context::Error>
    {
        let mut pop_operator = false;
        if let Some(operator) = operator {
            self.stack_trace.push(operator);
            pop_operator = true;
        }

        if let expr::Kind::Word(word) = expression.kind {
            if let Some(list) = self.functions.get(&word, &self.stack) {
                self.evaluate_list(
                    Some(expression.span),
                    &mut list.inner.into_iter(),
                )?;
            }
            else if let Some(ext) = self.extensions.get(&word, &self.stack) {
                ext(
                    self.host.clone(),
                    self,
                    expression.span,
                )?;
            }
            else if let Some(builtin) = self.builtins.get(&word, &self.stack) {
                builtin(self, expression.span)?;
            }
            else {
                return Err(
                    context::Error::UnknownFunction {
                        name: word,
                        span: expression.span,
                    }
                );
            }
        }
        else {
            self.stack.push::<expr::Any>(expression);
        }

        if pop_operator {
            self.stack_trace.pop();
        }

        Ok(())
    }

    fn evaluate_list(&mut self,
        operator:    Option<Span>,
        expressions: &mut dyn Iterator<Item=expr::Any>,
    )
        -> Result<(), context::Error>
    {
        for expression in expressions {
            self.evaluate_expr(operator.clone(), expression)?;
        }

        Ok(())
    }
}
