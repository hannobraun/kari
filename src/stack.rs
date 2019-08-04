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

    pub fn pop<T>(&mut self) -> Result<T, Error> where T: Type {
        match self.0.pop() {
            Some(expression) => T::check(expression),
            None             => Err(Error::StackEmpty),
        }
    }
}


pub trait Push {
    fn push(self, stack: &mut Vec<Expression>);
}

impl Push for Expression {
    fn push(self, stack: &mut Vec<Expression>) {
        stack.push(self)
    }
}


pub trait Type : Sized {
    fn check(_: Expression) -> Result<Self, Error>;
}

impl Type for Expression {
    fn check(expression: Expression) -> Result<Self, Error> {
        Ok(expression)
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

            impl Type for $type {
                fn check(expression: Expression) -> Result<Self, Error> {
                    match expression {
                        Expression::$type(expression) => {
                            Ok(expression)
                        }
                        expression => {
                            Err(Error::TypeError {
                                expected: $name,
                                actual:   expression,
                            })
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
        A: Push + Type + Clone,
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
        A: Push + Type + Clone,
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
        A: Push + Type + Clone,
        B: Push + Type + Clone,
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
