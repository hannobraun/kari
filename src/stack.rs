use std::fmt;

use crate::expression::{
    self,
    Expression,
};


pub struct Stack {
    substacks: Vec<Vec<Expression>>
}

impl Stack {
    pub fn new() -> Self {
        Self {
            substacks: vec![Vec::new()],
        }
    }

    pub fn push<T: Push>(&mut self, value: T::Data) {
        T::push(value, self)
    }

    pub fn pop<T: Pop>(&mut self) -> Result<T::Data, Error> {
        T::pop(self)
    }

    pub fn push_raw(&mut self, value: Expression) {
        let stack = self.substacks.last_mut().unwrap();
        stack.push(value)
    }

    pub fn pop_raw(&mut self) -> Option<Expression> {
        for stack in self.substacks.iter_mut().rev() {
            if let Some(value) = stack.pop() {
                return Some(value)
            }
        }

        None
    }

    pub fn create_substack(&mut self) {
        self.substacks.push(Vec::new());
    }

    pub fn destroy_substack(&mut self) -> Vec<Expression> {
        self.substacks.pop().unwrap()
    }
}


pub trait Push {
    type Data;

    fn push(_: Self::Data, _: &mut Stack);
}

pub trait Pop : Sized {
    type Data;

    fn pop(_: &mut Stack) -> Result<Self::Data, Error>;
}


impl<T> Push for T where T: expression::Into {
    type Data = T;

    fn push(data: Self::Data, stack: &mut Stack) {
        stack.push_raw(data.into_expression())
    }
}

impl<T> Pop for T where T: expression::From + expression::Name {
    type Data = T;

    fn pop(stack: &mut Stack) -> Result<Self::Data, Error> {
        match stack.pop_raw() {
            Some(expression) => {
                T::from_expression(expression)
                    .map_err(|expression|
                        Error::TypeError {
                            expected: T::NAME,
                            actual:   expression,
                        }
                    )
            }
            None => {
                Err(Error::StackEmpty {
                    expected: T::NAME,
                })
            }
        }
    }
}


impl<A, B> Push for (A, B)
    where
        A: Push<Data=A>,
        B: Push<Data=B>,
{
    type Data = Self;

    fn push(data: Self::Data, stack: &mut Stack) {
        stack.push::<A>(data.0);
        stack.push::<B>(data.1);
    }
}

impl<A, B> Pop for (A, B)
    where
        A: Pop<Data=A>,
        B: Pop<Data=B>,
{
    type Data = Self;

    fn pop(stack: &mut Stack) -> Result<Self::Data, Error> {
        let b = stack.pop::<B>()?;
        let a = stack.pop::<A>()?;
        Ok((a, b))
    }
}


#[derive(Debug)]
pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Expression,
    },
    StackEmpty {
        expected: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TypeError { expected, actual } => {
                write!(
                    f,
                    "Type error: Expected `{}`, found `{}`",
                    expected,
                    actual.kind,
                )?;
            }
            Error::StackEmpty { expected } => {
                write!(f, "Stack empty: Expected `{}`", expected)?;
            }
        }

        Ok(())
    }
}
