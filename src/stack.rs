use crate::parser::{
    Expression,
    List,
    Number,
};


pub struct Stack(Vec<Expression>);

impl Stack {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push<T: Push>(&mut self, value: T) {
        value.push(&mut self.0)
    }

    pub fn pop<T: Pop>(&mut self) -> Result<T, Error> {
        T::pop(&mut self.0)
    }
}


pub trait Push {
    fn push(self, stack: &mut Vec<Expression>);
}

pub trait Pop : Sized {
    fn pop(stack: &mut Vec<Expression>) -> Result<Self, Error>;
}


impl Push for Expression {
    fn push(self, stack: &mut Vec<Expression>) {
        stack.push(self)
    }
}

impl Pop for Expression {
    fn pop(stack: &mut Vec<Expression>) -> Result<Self, Error> {
        match stack.pop() {
            Some(expression) => Ok(expression),
            None             => Err(Error::StackEmpty),
        }
    }
}

macro_rules! impl_push_pop {
    ($($type:ident, $name:expr;)*) => {
        $(
            impl Push for $type {
                fn push(self, stack: &mut Vec<Expression>) {
                    stack.push(Expression::$type(self))
                }
            }

            impl Pop for $type {
                fn pop(stack: &mut Vec<Expression>) -> Result<Self, Error> {
                    match Expression::pop(stack) {
                        Ok(Expression::$type(expression)) => {
                            Ok(expression)
                        }
                        Ok(expression) => {
                            Err(Error::TypeError {
                                expected: $name,
                                actual:   expression,
                            })
                        }
                        Err(error) => {
                            Err(error)
                        }
                    }
                }
            }
        )*
    }
}

impl_push_pop!(
    List,   "list";
    Number, "number";
);


impl<A, B> Push for (A, B)
    where
        A: Push,
        B: Push,
{
    fn push(self, stack: &mut Vec<Expression>) {
        self.0.push(stack);
        self.1.push(stack);
    }
}

impl<A, B> Pop for (A, B)
    where
        A: Pop,
        B: Pop,
{
    fn pop(stack: &mut Vec<Expression>) -> Result<Self, Error> {
        let b = B::pop(stack)?;
        let a = A::pop(stack)?;
        Ok((a, b))
    }
}


#[derive(Debug)]
pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Expression,
    },
    StackEmpty,
}
