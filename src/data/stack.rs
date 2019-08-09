use std::fmt;

use crate::data::{
    expression::{
        self,
        Expression,
        Name as _,
    },
    span::Span,
};


#[derive(Debug)]
pub struct Stack {
    substacks: Vec<Vec<Expression>>
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

    pub fn push_raw(&mut self, value: Expression) {
        let stack = self.substacks.last_mut().unwrap();
        stack.push(value)
    }

    pub fn pop_raw(&mut self, operator: &Span) -> Result<Expression, Error> {
        for stack in self.substacks.iter_mut().rev() {
            if let Some(value) = stack.pop() {
                return Ok(value)
            }
        }

        Err(
            Error::StackEmpty {
                expected: Expression::NAME,
                operator: operator.clone(),
            }
        )
    }

    pub fn create_substack(&mut self) {
        self.substacks.push(Vec::new());
    }

    pub fn destroy_substack(&mut self) -> Vec<Expression> {
        self.substacks.pop().unwrap()
    }
}


pub trait Push {
    fn push(self, _: &mut Stack);
}

impl<T> Push for T
    where
        T: expression::Into,
{
    fn push(self, stack: &mut Stack) {
        stack.push_raw(self.into_expression())
    }
}

impl<A, B> Push for (A, B)
    where
        A: Push + expression::Into,
        B: Push + expression::Into,
{
    fn push(self, stack: &mut Stack) {
        stack.push(self.0);
        stack.push(self.1);
    }
}


pub trait Pop : Sized {
    fn pop(_: &mut Stack, operator: &Span) -> Result<Self, Error>;
}

impl Pop for Expression {
    fn pop(stack: &mut Stack, operator: &Span) -> Result<Self, Error> {
        stack.pop_raw(operator)
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
    Expression(expression::Error),
}

impl Error {
    pub fn span(self) -> Option<Span> {
        match self {
            Error::StackEmpty { operator, .. } => Some(operator),
            Error::Expression(error)           => error.span(),
        }
    }
}

impl From<expression::Error> for Error {
    fn from(from: expression::Error) -> Self {
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
