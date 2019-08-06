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

    pub fn push<T: Push>(&mut self, value: T) {
        value.push(self)
    }

    pub fn pop<T: Pop>(&mut self) -> Result<T, Error> {
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
    fn push(self, stack: &mut Stack);
}

pub trait Pop : Sized {
    fn pop(stack: &mut Stack) -> Result<Self, Error>;
}


impl<T> Push for T where T: expression::Into {
    fn push(self, stack: &mut Stack) {
        stack.push_raw(self.into_expression())
    }
}

impl<T> Pop for T where T: expression::Kind {
    fn pop(stack: &mut Stack) -> Result<Self, Error> {
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
        A: Push,
        B: Push,
{
    fn push(self, stack: &mut Stack) {
        stack.push(self.0);
        stack.push(self.1);
    }
}

impl<A, B> Pop for (A, B)
    where
        A: Pop,
        B: Pop,
{
    fn pop(stack: &mut Stack) -> Result<Self, Error> {
        let b = stack.pop()?;
        let a = stack.pop()?;
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
                    actual.data,
                )?;
            }
            Error::StackEmpty { expected } => {
                write!(f, "Stack empty: Expected `{}`", expected)?;
            }
        }

        Ok(())
    }
}
