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
        expr::{
            self,
            Expr as _,
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
        Stage as _,
        parser,
    },
    scope::{
        Function,
        Host,
        Scope,
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
            name:    Cow<str>,
        mut prelude: Box<dyn Stream>,
        mut program: Box<dyn Stream>,
        mut scope:   Scope<Function<H>>,
    )
        -> bool
    {
        let prelude_name = "<prelude>";

        let prelude_pipeline = pipeline::new(
            prelude_name.into(),
            &mut prelude,
        );
        self.evaluate_expressions(prelude_pipeline, &mut scope)
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

        let result = self.evaluate_expressions(pipeline, &mut scope);
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
        mut parser: Parser,
            scope:  &mut Scope<Function<H>>,
    )
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
                scope,
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

        Ok(expr::List::new(expressions, start.merge(&end)))
    }

    fn evaluate_expr(&mut self,
        scope:    &mut Scope<Function<H>>,
        operator: Option<Span>,
        expr:     expr::Any,
    )
        -> Result<(), context::Error>
    {
        let mut pop_operator = false;
        if let Some(operator) = operator {
            self.stack_trace.push(operator);
            pop_operator = true;
        }

        if let expr::Kind::Word(word) = expr.kind {
            match scope.get(&word, &self.stack) {
                Ok(f) => {
                    match f {
                        Function::Builtin(f) => {
                            f(self.host.clone(), self, scope, expr.span)?;
                        }
                        Function::UserDefined(f) => {
                            self.evaluate_list(scope, Some(expr.span), f)?;
                        }
                    }
                }
                Err(_) => {
                    return Err(
                        context::Error::FunctionNotFound {
                            name: word,
                            span: expr.span,
                        }
                    );
                }
            }
        }
        else {
            self.stack.push::<expr::Any>(expr);
        }

        if pop_operator {
            self.stack_trace.pop();
        }

        Ok(())
    }

    fn evaluate_list(&mut self,
        scope:    &mut Scope<Function<H>>,
        operator: Option<Span>,
        list:     expr::List,
    )
        -> Result<(), context::Error>
    {
        for expr in list {
            self.evaluate_expr(scope, operator.clone(), expr)?;
        }

        Ok(())
    }
}
