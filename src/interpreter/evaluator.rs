use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io,
};

use crate::{
    builtins::builtins,
    context::{
        self,
        Context,
    },
    data::{
        expression::Expression,
        functions::{
            self,
            Functions,
            Scope,
        },
        span::Span,
        stack::Stack,
        value::{
            self,
            Value as _,
        },
    },
    function::Function,
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

    functions:  Functions<Function<Host>>,
    stack:      Stack,
    call_stack: Vec<Span>,
}

impl<Host> Evaluator<Host> {
    pub fn new(
        stdout: Box<dyn io::Write>,
        stderr: Box<dyn io::Write>,
    )
        -> Self
    {
        let mut functions = Functions::new();
        builtins(&mut functions);

        Self {
            streams: HashMap::new(),
            stdout,
            stderr,

            functions,
            stack:      Stack::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn run(mut self,
            host:      &mut Host,
            name:      Cow<str>,
        mut prelude:   Box<dyn Stream>,
        mut program:   Box<dyn Stream>,
    )
        -> bool
    {
        let prelude_name = "<prelude>";

        let prelude_pipeline = pipeline::new(
            prelude_name.into(),
            &mut prelude,
        );

        self
            .evaluate_expressions(
                host,
                self.functions.root_scope(),
                prelude_pipeline,
            )
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

        let result = self.evaluate_expressions(
            host,
            self.functions.root_scope(),
            pipeline,
        );
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
            host:   &mut Host,
            scope:  Scope,
        mut parser: Parser,
    )
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
                            stack_trace: self.call_stack.clone(),
                        }
                    );
                }
            };

            let list_scope = self.functions.new_scope(scope, "list");

            let result = self.evaluate_value(
                host,
                scope,
                value::Any::from_expression(
                    expression,
                    list_scope,
                ),
            );
            if let Err(error) = result {
                return Err(
                    Error {
                        kind:        error.into(),
                        stack_trace: self.call_stack.clone(),
                    }
                );
            }
        }
    }
}

impl<Host> Context<Host> for Evaluator<Host> {
    fn functions(&mut self) -> &mut Functions<Function<Host>> {
        &mut self.functions
    }

    fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    fn output(&mut self) -> &mut dyn io::Write {
        &mut self.stdout
    }

    fn load(&mut self, name: value::String, scope: Scope)
        -> Result<value::List, context::Error>
    {
        let module_scope = self.functions.new_scope(scope, name.inner.clone());

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

        Ok(
            value::List::new(
                value::ListInner::from_expressions(
                    expressions,
                    module_scope,
                ),
                start.merge(&end),
            )
        )
    }

    fn evaluate_value(&mut self,
        host:  &mut Host,
        scope: Scope,
        value: value::Any,
    )
        -> Result<(), context::Error>
    {
        if let value::Kind::Word(word) = value.kind {
            self.call_stack.push(value.span.clone());

            match self.functions.get(scope, &word, &self.stack) {
                Ok(f) => {
                    match f {
                        Function::Builtin(f) => {
                            f(
                                host,
                                self,
                                scope,
                                value.span,
                            )?;
                        }
                        Function::UserDefined { body } => {
                            self.evaluate_list(
                                host,
                                body,
                            )?;
                        }
                    }
                }
                Err(functions::GetError { candidates, scope, .. }) => {
                    return Err(
                        context::Error::FunctionNotFound {
                            name:  word,
                            stack: self.stack.clone(),
                            candidates,
                            scope: scope,
                        }
                    );
                }
            }

            self.call_stack.pop();
        }
        else {
            self.stack.push::<value::Any>(value);
        }

        Ok(())
    }

    fn evaluate_list(&mut self,
        host: &mut Host,
        list: value::List,
    )
        -> Result<(), context::Error>
    {
        let scope = self.functions().root_scope();

        for expr in list {
            self.evaluate_value(
                host,
                scope,
                expr,
            )?;
        }

        Ok(())
    }
}
