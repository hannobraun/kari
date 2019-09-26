use std::fmt;

use crate::data::{
    span::Span,
    types::{
        self,
        Type,
        TypeError,
    },
    value::{
        self,
        Value,
    },
};


#[derive(Clone, Debug)]
pub struct Stack {
    substacks: Vec<Vec<value::Any>>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            substacks: vec![Vec::new()],
        }
    }

    pub fn push<T: Push>(&mut self, value: T) -> &mut Self {
        T::push(value, self);
        self
    }

    pub fn pop<T: Pop>(&mut self, ty: T)
        -> Result<T::Value, Error>
    {
        ty.pop(self)
    }

    pub fn peek(&self) -> impl Iterator<Item=&value::Any> + '_ {
        self.substacks.iter().flatten().rev()
    }

    pub fn push_raw(&mut self, value: value::Any) {
        let stack = self.substacks.last_mut().unwrap();
        stack.push(value)
    }

    pub fn pop_raw(&mut self) -> Result<value::Any, Error> {
        for stack in self.substacks.iter_mut().rev() {
            if let Some(value) = stack.pop() {
                return Ok(value)
            }
        }

        Err(
            Error::StackEmpty {
                expected: types::Any.name(),
            }
        )
    }

    pub fn create_substack(&mut self) {
        self.substacks.push(Vec::new());
    }

    pub fn destroy_substack(&mut self) -> Vec<value::Any> {
        self.substacks.pop().unwrap()
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for substack in &self.substacks {
            for expr in substack {
                write!(f, "{} ", expr.kind)?;
            }
        }

        Ok(())
    }
}


pub trait Push {
    fn push(self, _: &mut Stack);
}

impl<T> Push for T
    where
        T: Value,
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
    type Value;

    fn pop(&self, _: &mut Stack) -> Result<Self::Value, Error>;
}

impl<T> Pop for &T where T: types::Downcast {
    type Value = T::Value;

    fn pop(&self, stack: &mut Stack)
        -> Result<Self::Value, Error>
    {
        let expr = stack.pop_raw()?;
        Ok(self.downcast(expr)?)
    }
}

impl<A, B> Pop for (A, B)
    where
        A: Pop + Copy,
        B: Pop + Copy,
{
    type Value = (A::Value, B::Value);

    fn pop(&self, stack: &mut Stack)
        -> Result<Self::Value, Error>
    {
        let b = stack.pop(self.1)?;
        let a = stack.pop(self.0)?;
        Ok((a, b))
    }
}


#[derive(Debug)]
pub enum Error {
    StackEmpty {
        expected: &'static str,
    },
    Type(TypeError),
}

impl Error {
    pub fn spans<'r>(&'r self, spans: &mut Vec<&'r Span>) {
        match self {
            Error::StackEmpty { .. } => (),
            Error::Type(error)       => error.spans(spans),
        }
    }
}

impl From<TypeError> for Error {
    fn from(from: TypeError) -> Self {
        Error::Type(from)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::StackEmpty { expected, .. } => {
                write!(f, "Stack empty: Expected `{}`", expected)?;
            }
            Error::Type(error) => {
                error.fmt(f)?;
            }
        }

        Ok(())
    }
}
