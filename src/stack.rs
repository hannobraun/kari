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

    pub fn push<T>(&mut self, value: T) where T: Type {
        self.0.push(value.to_expression())
    }

    pub fn pop<T>(&mut self) -> Result<T, Error> where T: Type {
        match self.0.pop() {
            Some(expression) => T::check(expression),
            None             => Err(Error::StackEmpty),
        }
    }
}


pub trait Type : Sized {
    fn check(_: Expression) -> Result<Self, Error>;
    fn to_expression(self) -> Expression;
}

impl Type for Expression {
    fn check(expression: Expression) -> Result<Self, Error> {
        Ok(expression)
    }

    fn to_expression(self) -> Expression {
        self
    }
}

macro_rules! impl_type {
    ($($type:ident, $name:expr;)*) => {
        $(
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

                fn to_expression(self) -> Expression {
                    Expression::$type(self)
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
        A: Type + Clone,
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
        A: Type + Clone,
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
        A: Type + Clone,
        B: Type + Clone,
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
