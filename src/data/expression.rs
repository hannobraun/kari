use std::{
    fmt,
    ops::{
        Add,
        Mul,
        Not,
    },
};

use crate::data::span::{
    Span,
    WithSpan,
};


#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: Kind,
    pub span: Span,
}

impl Expression {
    pub fn check<T>(self) -> Result<WithSpan<T>, Error>
        where
            WithSpan<T>: From + Name,
    {
        WithSpan::<T>::from_expression(self)
            .map_err(|expression|
                Error::TypeError {
                    expected: WithSpan::<T>::NAME,
                    actual:   expression,
                }
            )
    }
}


pub struct E2(pub Expression, pub Expression);

impl E2 {
    pub fn check<A, B>(self) -> Result<(WithSpan<A>, WithSpan<B>), Error>
        where
            WithSpan<A>: From + Name,
            WithSpan<B>: From + Name,
    {
        Ok((self.0.check()?, self.1.check()?))
    }
}


#[derive(Clone, Debug)]
pub enum Kind {
    Bool(Bool),
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Bool(b)        => b.0.fmt(f),
            Kind::Number(number) => number.0.fmt(f),
            Kind::List(list)     => list.fmt(f),
            Kind::String(string) => string.fmt(f),
            Kind::Word(word)     => word.fmt(f),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Bool(pub bool);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Number(pub u32);

#[derive(Clone, Debug)]
pub struct List(pub Vec<Expression>);


impl Not for Bool {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bool(self.0.not())
    }
}


impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0 + rhs.0)
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Number(self.0 * rhs.0)
    }
}


impl List {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl IntoIterator for List {
    type Item     = <Vec<Expression> as IntoIterator>::Item;
    type IntoIter = <Vec<Expression> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for item in &self.0 {
            write!(f, "{} ", item.kind)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}


pub trait Name {
    const NAME: &'static str;
}

pub trait Into {
    fn into_expression(self) -> Expression;
}

pub trait From : Sized {
    fn from_expression(expression: Expression) -> Result<Self, Expression>;
}


impl Name for Expression {
    const NAME: &'static str = "expression";
}

impl Into for Expression {
    fn into_expression(self) -> Expression {
        self
    }
}

impl From for Expression {
    fn from_expression(expression: Expression) -> Result<Self, Expression> {
        Ok(expression)
    }
}


macro_rules! impl_expression {
    ($($ty:ident, $name:expr;)*) => {
        $(
            impl Name for WithSpan<$ty> {
                const NAME: &'static str = $name;
            }

            impl Into for WithSpan<$ty> {
                fn into_expression(self) -> Expression {
                    Expression {
                        kind: Kind::$ty(self.value),
                        span: self.span,
                    }
                }
            }

            impl From for WithSpan<$ty> {
                fn from_expression(expression: Expression)
                    -> Result<Self, Expression>
                {
                    match expression.kind {
                        Kind::$ty(value) => {
                            Ok(
                                WithSpan {
                                    value,
                                    span: expression.span,
                                }
                            )
                        }
                        _ => {
                            Err(expression)
                        }
                    }
                }
            }
        )*
    }
}

impl_expression!(
    Bool,   "bool";
    List,   "list";
    Number, "number";
    String, "string";
);


#[derive(Debug)]
pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Expression,
    },
}

impl Error {
    pub fn span(self) -> Option<Span> {
        match self {
            Error::TypeError { actual, .. } => Some(actual.span),
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
                )
            }
        }
    }
}
