use std::fmt;

use crate::data::{
    expression::{
        self,
        Expression,
        From as _,
        Name as _,
    },
    span::{
        Span,
        WithSpan,
    },
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

    pub fn push<T: Push>(&mut self, value: T) {
        T::push(value, self)
    }

    pub fn pop<T: Pop>(&mut self, operator: &Span) -> Result<T::Data, Error> {
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
    fn push(self, _: &mut Stack);
}

impl<T> Push for T
    where
        T: expression::Into,
{
    fn push(self, stack: &mut Stack) {
        stack.push_raw(self.into_expression())
    }
}

impl<A, B> Push for (A, B)
    where
        A: Push + expression::Into,
        B: Push + expression::Into,
{
    fn push(self, stack: &mut Stack) {
        stack.push(self.0);
        stack.push(self.1);
    }
}


pub trait Pop : Sized {
    type Data;

    fn pop(_: &mut Stack, operator: &Span) -> Result<Self::Data, Error>;
}

impl Pop for Expression {
    type Data = Expression;

    fn pop(stack: &mut Stack, operator: &Span) -> Result<Self::Data, Error> {
        stack.pop_raw()
            .ok_or_else(|| {
                Error::StackEmpty {
                    expected: Expression::NAME,
                    operator: operator.clone(),
                }
            })
    }
}

impl<T> Pop for T
    where WithSpan<T>: expression::From + expression::Name
{
    type Data = WithSpan<T>;

    fn pop(stack: &mut Stack, operator: &Span) -> Result<Self::Data, Error> {
        match stack.pop_raw() {
            Some(expression) => {
                WithSpan::<T>::from_expression(expression)
                    .map_err(|expression|
                        Error::TypeError {
                            expected: WithSpan::<T>::NAME,
                            actual:   expression,
                        }
                    )
            }
            None => {
                Err(Error::StackEmpty {
                    expected: WithSpan::<T>::NAME,
                    operator: operator.clone(),
                })
            }
        }
    }
}

impl<A, B> Pop for (A, B)
    where
        A:           Pop<Data=WithSpan<A>>,
        B:           Pop<Data=WithSpan<B>>,
        WithSpan<A>: expression::From + expression::Name,
        WithSpan<B>: expression::From + expression::Name,
{
    type Data = (WithSpan<A>, WithSpan<B>);

    fn pop(stack: &mut Stack, operator: &Span) -> Result<Self::Data, Error> {
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
    pub fn span(self) -> Option<Span> {
        match self {
            Error::TypeError { actual, .. }    => Some(actual.span),
            Error::StackEmpty { operator, .. } => Some(operator),
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
