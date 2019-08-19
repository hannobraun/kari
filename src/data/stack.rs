use std::fmt;

use crate::data::{
    expr::{
        self,
        Expr,
    },
    span::Span,
    types::Type,
};


#[derive(Debug)]
pub struct Stack {
    substacks: Vec<Vec<expr::Any>>
}

impl Stack {
    pub fn new() -> Self {
        Self {
            substacks: vec![Vec::new()],
        }
    }

    pub fn push<T: Push>(&mut self, value: T) {
        T::push(value, self)
    }

    pub fn pop<T: Pop>(&mut self, operator: &Span) -> Result<T, Error> {
        T::pop(self, operator)
    }

    pub fn push_raw(&mut self, value: expr::Any) {
        let stack = self.substacks.last_mut().unwrap();
        stack.push(value)
    }

    pub fn pop_raw(&mut self, operator: &Span) -> Result<expr::Any, Error> {
        for stack in self.substacks.iter_mut().rev() {
            if let Some(value) = stack.pop() {
                return Ok(value)
            }
        }

        Err(
            Error::StackEmpty {
                expected: expr::Any::NAME,
                operator: operator.clone(),
            }
        )
    }

    pub fn create_substack(&mut self) {
        self.substacks.push(Vec::new());
    }

    pub fn destroy_substack(&mut self) -> Vec<expr::Any> {
        self.substacks.pop().unwrap()
    }
}


pub trait Push {
    fn push(self, _: &mut Stack);
}

impl<T> Push for T
    where
        T: Expr,
{
    fn push(self, stack: &mut Stack) {
        stack.push_raw(self.into_any())
    }
}

impl<A, B> Push for (A, B)
    where
        A: Push,
        B: Push,
{
    fn push(self, stack: &mut Stack) {
        stack.push(self.0);
        stack.push(self.1);
    }
}


pub trait Pop : Sized {
    fn pop(_: &mut Stack, operator: &Span) -> Result<Self, Error>;
}

impl<T> Pop for T where T: Expr + Type<Value=Self> {
    fn pop(stack: &mut Stack, operator: &Span) -> Result<Self, Error> {
        Ok(stack.pop_raw(operator)?.check::<T>()?)
    }
}

impl<A, B> Pop for (A, B)
    where
        A: Pop,
        B: Pop,
{
    fn pop(stack: &mut Stack, operator: &Span) -> Result<Self, Error> {
        let b = stack.pop::<B>(operator)?;
        let a = stack.pop::<A>(operator)?;
        Ok((a, b))
    }
}


#[derive(Debug)]
pub enum Error {
    StackEmpty {
        expected: &'static str,
        operator: Span,
    },
    Expression(expr::Error),
}

impl Error {
    pub fn spans(self, spans: &mut Vec<Span>) {
        match self {
            Error::StackEmpty { operator, .. } => spans.push(operator),
            Error::Expression(error)           => error.spans(spans),
        }
    }
}

impl From<expr::Error> for Error {
    fn from(from: expr::Error) -> Self {
        Error::Expression(from)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::StackEmpty { expected, .. } => {
                write!(f, "Stack empty: Expected `{}`", expected)?;
            }
            Error::Expression(error) => {
                error.fmt(f)?;
            }
        }

        Ok(())
    }
}
