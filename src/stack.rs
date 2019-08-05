use crate::parser::{
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

    pub fn pop_raw(&mut self) -> Result<Expression, Error> {
        for stack in self.substacks.iter_mut().rev() {
            if let Some(value) = stack.pop() {
                return Ok(value)
            }
        }

        Err(Error::StackEmpty)
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
    StackEmpty,
}
