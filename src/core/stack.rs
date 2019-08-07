use std::fmt;

use crate::core::{
    expression::{
        self,
        Expression,
        From as _,
        Into as _,
        Name as _,
    },
    span::Span,
};


#[derive(Debug)]
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

    pub fn pop<T: Pop>(&mut self, operator: Span) -> Result<T::Data, Error> {
        T::pop(self, operator)
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

    fn pop(_: &mut Stack, operator: Span) -> Result<Self::Data, Error>;
}


impl Push for Expression {
    type Data = Expression;

    fn push(data: Self::Data, stack: &mut Stack) {
        stack.push_raw(data)
    }
}

impl Pop for Expression {
    type Data = Expression;

    fn pop(stack: &mut Stack, operator: Span) -> Result<Self::Data, Error> {
        stack.pop_raw()
            .ok_or(
                Error::StackEmpty {
                    expected: Expression::NAME,
                    operator,
                }
            )
    }
}


impl<T> Push for T
    where expression::Data<T>: expression::Into
{
    type Data = expression::Data<T>;

    fn push(data: Self::Data, stack: &mut Stack) {
        stack.push_raw(data.into_expression())
    }
}

impl<T> Pop for T
    where expression::Data<T>: expression::From + expression::Name
{
    type Data = expression::Data<T>;

    fn pop(stack: &mut Stack, operator: Span) -> Result<Self::Data, Error> {
        match stack.pop_raw() {
            Some(expression) => {
                expression::Data::<T>::from_expression(expression)
                    .map_err(|expression|
                        Error::TypeError {
                            expected: expression::Data::<T>::NAME,
                            actual:   expression,
                        }
                    )
            }
            None => {
                Err(Error::StackEmpty {
                    expected: expression::Data::<T>::NAME,
                    operator,
                })
            }
        }
    }
}


impl<A, B> Push for (A, B)
    where
        expression::Data<A>: Push<Data=expression::Data<A>> + expression::Into,
        expression::Data<B>: Push<Data=expression::Data<B>> + expression::Into,
{
    type Data = (expression::Data<A>, expression::Data<B>);

    fn push(data: Self::Data, stack: &mut Stack) {
        stack.push::<A>(data.0);
        stack.push::<B>(data.1);
    }
}

impl<A, B> Pop for (A, B)
    where
        A:                   Pop<Data=expression::Data<A>>,
        B:                   Pop<Data=expression::Data<B>>,
        expression::Data<A>: expression::From + expression::Name,
        expression::Data<B>: expression::From + expression::Name,
{
    type Data = (expression::Data<A>, expression::Data<B>);

    fn pop(stack: &mut Stack, operator: Span) -> Result<Self::Data, Error> {
        let b = stack.pop::<B>(operator)?;
        let a = stack.pop::<A>(operator)?;
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
        operator: Span,
    },
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::TypeError { actual, .. }    => Some(actual.span),
            Error::StackEmpty { operator, .. } => Some(*operator),
        }
    }
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
            Error::StackEmpty { expected, .. } => {
                write!(f, "Stack empty: Expected `{}`", expected)?;
            }
        }

        Ok(())
    }
}
