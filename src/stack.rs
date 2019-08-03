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

    pub fn push(&mut self, expression: Expression) {
        self.0.push(expression)
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
}

impl Type for Expression {
    fn check(expression: Expression) -> Result<Self, Error> {
        Ok(expression)
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
            }
        )*
    }
}

impl_type!(
    List,   "list";
    Number, "number";
);


#[derive(Debug)]
pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Expression,
    },
    StackEmpty,
}
