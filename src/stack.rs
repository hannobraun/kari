use std::fmt;

use crate::{
    source::Span,
    value::{self, cast::TypeError, Value},
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

    pub fn pop<T: Pop>(&mut self) -> Result<T, Error> {
        T::pop(self)
    }

    pub fn peek(&self) -> impl Iterator<Item = &value::Any> + '_ {
        self.substacks.iter().flatten().rev()
    }

    pub fn push_raw(&mut self, value: value::Any) {
        let stack = self.substacks.last_mut().unwrap();
        stack.push(value)
    }

    pub fn pop_raw(&mut self) -> Option<value::Any> {
        for stack in self.substacks.iter_mut().rev() {
            if let Some(value) = stack.pop() {
                return Some(value);
            }
        }

        None
    }

    pub fn create_substack(&mut self) {
        self.substacks.push(Vec::new());
    }

    pub fn destroy_substack(&mut self) -> Vec<value::Any> {
        self.substacks.pop().unwrap()
    }

    pub fn into_vec(mut self) -> Vec<value::Any> {
        let mut vec = Vec::new();

        while let Some(value) = self.pop_raw() {
            vec.insert(0, value);
        }

        vec
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
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

pub trait Pop: Sized {
    fn pop(_: &mut Stack) -> Result<Self, Error>;
}

impl Pop for value::Any {
    fn pop(stack: &mut Stack) -> Result<Self, Error> {
        stack.pop_raw().ok_or(Error::StackEmpty)
    }
}

impl Pop for (value::Any, value::Any) {
    fn pop(stack: &mut Stack) -> Result<Self, Error> {
        let b = stack.pop()?;
        let a = stack.pop()?;
        Ok((a, b))
    }
}

impl Pop for (value::Any, value::Any, value::Any) {
    fn pop(stack: &mut Stack) -> Result<Self, Error> {
        let c = stack.pop()?;
        let b = stack.pop()?;
        let a = stack.pop()?;
        Ok((a, b, c))
    }
}

#[derive(Debug)]
pub enum Error {
    StackEmpty,
    TypeError(TypeError),
}

impl Error {
    pub fn spans<'r>(&'r self, spans: &mut Vec<&'r Span>) {
        match self {
            Error::StackEmpty => (),
            Error::TypeError(err) => err.spans(spans),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::StackEmpty => {
                write!(f, "Tried to pop value, but stack is empty")
            }

            Error::TypeError(error) => error.fmt(f),
        }
    }
}
