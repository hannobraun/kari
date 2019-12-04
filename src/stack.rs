use std::fmt;

use crate::{
    token::Span,
    value::{
        self,
        Value,
        types::{
            self,
            TypeError,
        },
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

    pub fn pop<T: Pop>(&mut self, ty: T) -> Result<T::Value, Error> {
        ty.pop(self)
    }

    pub fn peek(&self) -> impl Iterator<Item=&value::Any> + '_ {
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

    fn pop(&self, stack: &mut Stack) -> Result<Self::Value, Error> {
        let expr = stack.pop_raw()
            .ok_or(Error::StackEmpty)?;

        let value = self.downcast(expr)
            .map_err(|err| Error::TypeError(err))?;

        Ok(value)
    }
}

impl<A, B> Pop for (A, B)
    where
        A: Pop + Copy,
        B: Pop + Copy,
{
    type Value = (A::Value, B::Value);

    fn pop(&self, stack: &mut Stack) -> Result<Self::Value, Error> {
        let b = stack.pop(self.1)?;
        let a = stack.pop(self.0)?;
        Ok((a, b))
    }
}

impl<A, B, C> Pop for (A, B, C)
    where
        A: Pop + Copy,
        B: Pop + Copy,
        C: Pop + Copy,
{
    type Value = (A::Value, B::Value, C::Value);

    fn pop(&self, stack: &mut Stack) -> Result<Self::Value, Error> {
        let c = stack.pop(self.2)?;
        let b = stack.pop(self.1)?;
        let a = stack.pop(self.0)?;
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
            Error::StackEmpty     => (),
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
