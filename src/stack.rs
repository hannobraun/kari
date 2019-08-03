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


#[derive(Debug)]
pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Expression,
    },
    StackEmpty,
}
