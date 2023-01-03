pub mod error;
pub mod stream;

use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Cursor},
};

use parser::Parser;

use crate::{
    builtins::builtins,
    call_stack::{CallStack, StackFrame},
    context::{self, Context},
    functions::{self, Builtin, DefineError, Function, Functions, Scope},
    pipeline::{parser, tokenizer::span::Merge, Pipeline},
    prelude::*,
    stack::Stack,
    value::{self, types::Type, v},
};

use self::{error::Error, stream::Stream};

pub struct Interpreter<Host> {
    streams: HashMap<String, Box<dyn Stream>>,
    stdout: Box<dyn io::Write>,
    stderr: Box<dyn io::Write>,

    functions: Functions<Function<Host>>,
    stack: Stack,
    call_stack: CallStack,
}

impl<Host> Interpreter<Host> {
    pub fn new(stdout: Box<dyn io::Write>, stderr: Box<dyn io::Write>) -> Self {
        Self {
            streams: HashMap::new(),
            stdout,
            stderr,

            functions: Functions::new(),
            stack: Stack::new(),
            call_stack: CallStack::new(),
        }
    }

    pub fn with_default_builtins(mut self) -> Self {
        builtins(&mut self.functions);
        self
    }

    pub fn with_default_prelude(
        mut self,
        host: &mut Host,
    ) -> Result<Self, Error> {
        let name = "<prelude>";
        let mut prelude =
            Cursor::new(&include_bytes!("../kr/src/prelude.kr")[..]);

        let mut prelude_pipeline = Pipeline::new(name.into(), &mut prelude);

        self.evaluate_expressions(
            host,
            self.functions.root_scope(),
            prelude_pipeline.parser,
            &mut prelude_pipeline.source,
        )?;

        // We panic on errors in the prelude itself, but errors in other modules
        // might still produce stack traces with spans that refer to the
        // prelude.
        self.streams.insert(name.into(), Box::new(prelude));

        Ok(self)
    }

    pub fn with_default_modules(mut self) -> Self {
        self.streams.insert(
            "std".into(),
            Box::new(Cursor::new(&include_bytes!("../kr/src/std.kr")[..])),
        );

        self
    }

    pub fn with_builtin(
        mut self,
        name: &str,
        args: &[&'static dyn Type],
        builtin: Builtin<Host>,
    ) -> Result<Self, DefineError> {
        self.functions.define(
            self.functions.root_scope(),
            String::from(name),
            args,
            Function::Builtin(builtin),
        )?;
        Ok(self)
    }

    pub fn run(
        mut self,
        host: &mut Host,
        name: Cow<str>,
        mut program: Box<dyn Stream>,
    ) -> Result<Vec<value::Any>, Error> {
        let mut pipeline =
            Pipeline::new(name.clone().into_owned(), &mut program);

        let result = self.evaluate_expressions(
            host,
            self.functions.root_scope(),
            pipeline.parser,
            &mut pipeline.source,
        );
        if let Err(error) = result {
            self.streams.insert(name.into_owned(), program);

            if let Err(error) = error.print(&mut self.streams, &mut self.stderr)
            {
                println!("Error printing error: {}", error)
            }

            return Err(error);
        }

        Ok(self.stack.into_vec())
    }

    fn evaluate_expressions<R>(
        &mut self,
        host: &mut Host,
        scope: Scope,
        mut parser: Parser<R>,
        source: &mut String,
    ) -> Result<(), Error>
    where
        R: io::Read,
    {
        loop {
            let expression = match parser.next_expression(source) {
                Ok(expression) => expression,
                Err(parser::Error::EndOfStream) => {
                    return Ok(());
                }
                Err(error) => {
                    return Err(Error {
                        kind: error.into(),
                        call_stack: self.call_stack.clone(),
                    });
                }
            };

            let list_scope = self.functions.new_scope(scope, "list");

            let result = self.evaluate_value(
                host,
                scope,
                value::Any::from_expression(expression, list_scope),
            );
            if let Err(error) = result {
                return Err(Error {
                    kind: error.into(),
                    call_stack: self.call_stack.clone(),
                });
            }
        }
    }
}

impl<Host> Context<Host> for Interpreter<Host> {
    fn functions(&mut self) -> &mut Functions<Function<Host>> {
        &mut self.functions
    }

    fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    fn call_stack(&mut self) -> &mut CallStack {
        &mut self.call_stack
    }

    fn output(&mut self) -> &mut dyn io::Write {
        &mut self.stdout
    }

    fn load(
        &mut self,
        name: v::String,
        scope: Scope,
    ) -> Result<v::List, context::Error> {
        let name = name.inner;

        let module_scope = self.functions.new_scope(scope, name.clone());

        let stream = match self.streams.get_mut(&name) {
            Some(stream) => stream,
            None => return Err(context::Error::ModuleNotFound(name)),
        };

        let mut pipeline = Pipeline::new(name, stream.as_mut());
        let mut expressions = Vec::new();

        loop {
            match pipeline.parser.next_expression(&mut pipeline.source) {
                Ok(expression) => expressions.push(expression),
                Err(parser::Error::EndOfStream) => break,
                Err(error) => return Err(error.into()),
            }
        }

        let start = expressions
            .first()
            .and_then(|expression| expression.span.clone());
        let end = expressions
            .last()
            .and_then(|expression| expression.span.clone());

        Ok(v::List::new(
            value::ListInner::from_expressions(expressions, module_scope),
            start.merge(end),
        ))
    }

    fn evaluate_value(
        &mut self,
        host: &mut Host,
        scope: Scope,
        value: value::Any,
    ) -> Result<(), context::Error> {
        if let value::Kind::Word(word) = value.kind {
            self.call_stack.frames.push(StackFrame {
                scope,
                span: value.span,
            });

            match self.functions.get(scope, &word, &self.stack) {
                Ok(f) => match f {
                    Function::Builtin(f) => {
                        f(host, self, scope)?;
                    }
                    Function::UserDefined { body } => {
                        self.evaluate_list(host, body)?;
                    }
                },
                Err(functions::GetError {
                    candidates, scope, ..
                }) => {
                    return Err(context::Error::FunctionNotFound {
                        name: word,
                        stack: self.stack.clone(),
                        candidates,
                        scope,
                    });
                }
            }

            self.call_stack.frames.pop();
        } else {
            self.stack.push::<value::Any>(value);
        }

        Ok(())
    }

    fn evaluate_list(
        &mut self,
        host: &mut Host,
        list: v::List,
    ) -> Result<(), context::Error> {
        let scope = list.inner.scope;

        for expr in list {
            self.evaluate_value(host, scope, expr)?;
        }

        Ok(())
    }
}
