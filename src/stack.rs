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

macro_rules! impl_type {
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

impl_type!(
    List,   "list";
    Number, "number";
);


pub trait Types {
    fn take(&mut self, _: &mut Stack) -> Result<(), Error>;
    fn place(&self, _: &mut Stack);
}

impl Types for () {
    fn take(&mut self, _: &mut Stack) -> Result<(), Error> {
        Ok(())
    }

    fn place(&self, _: &mut Stack) {
        ()
    }
}

impl<A> Types for A
    where
        A: Push + Pop + Clone,
{
    fn take(&mut self, stack: &mut Stack) -> Result<(), Error> {
        *self = stack.pop::<A>()?;

        Ok(())
    }

    fn place(&self, stack: &mut Stack) {
        stack.push(self.clone());
    }
}

impl<A> Types for (A,)
    where
        A: Push + Pop + Clone,
{
    fn take(&mut self, stack: &mut Stack) -> Result<(), Error> {
        self.0 = stack.pop::<A>()?;

        Ok(())
    }

    fn place(&self, stack: &mut Stack) {
        stack.push(self.0.clone());
    }
}

impl<A, B> Types for (A, B)
    where
        A: Push + Pop + Clone,
        B: Push + Pop + Clone,
{
    fn take(&mut self, stack: &mut Stack) -> Result<(), Error> {
        self.1 = stack.pop::<B>()?;
        self.0 = stack.pop::<A>()?;

        Ok(())
    }

    fn place(&self, stack: &mut Stack) {
        stack.push(self.0.clone());
        stack.push(self.1.clone());
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
