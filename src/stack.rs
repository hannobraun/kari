use std::fmt;

use crate::parser::{
    Bool,
    Expression,
    List,
    Number,
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


impl Push for Expression {
    fn push(self, stack: &mut Stack) {
        stack.push_raw(self)
    }
}

impl Pop for Expression {
    fn pop(stack: &mut Stack) -> Result<Self, Error> {
        stack.pop_raw()
            .ok_or(Error::StackEmpty { expected: "expression" })
    }
}

macro_rules! impl_push_pop {
    ($($type:ident, $name:expr;)*) => {
        $(
            impl Push for $type {
                fn push(self, stack: &mut Stack) {
                    stack.push(Expression::$type(self))
                }
            }

            impl Pop for $type {
                fn pop(stack: &mut Stack) -> Result<Self, Error> {
                    match stack.pop_raw() {
                        Some(Expression::$type(expression)) => {
                            Ok(expression)
                        }
                        Some(expression) => {
                            Err(Error::TypeError {
                                expected: $name,
                                actual:   expression,
                            })
                        }
                        None => {
                            Err(Error::StackEmpty {
                                expected: $name,
                            })
                        }
                    }
                }
            }
        )*
    }
}

impl_push_pop!(
    Bool,   "bool";
    List,   "list";
    Number, "number";
);


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
                    actual,
                )?;
            }
            Error::StackEmpty { expected } => {
                write!(f, "Stack empty: Expected `{}`", expected)?;
            }
        }

        Ok(())
    }
}
