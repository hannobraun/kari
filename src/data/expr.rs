use std::{
    cmp::Ordering,
    fmt,
    ops::{
        Add,
        Mul,
        Not,
    },
    string::String as StdString,
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


macro_rules! kinds {
    (
        $(
            $ty:ident,
            $name:expr,
            $inner:ty;
        )*
    ) => {
        #[derive(Clone, Debug)]
        pub enum Kind {
            $($ty($inner),)*
        }


        $(
            #[derive(Clone, Debug)]
            pub struct $ty {
                pub inner: $inner,
            }

            impl $ty {
                pub fn new(inner: $inner) -> Self {
                    Self {
                        inner,
                    }
                }
            }

            impl Name for WithSpan<$ty> {
                const NAME: &'static str = $name;
            }

            impl Into for WithSpan<$ty> {
                fn into_expression(self) -> Expression {
                    Expression {
                        kind: Kind::$ty(self.value.inner),
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
                                    value: $ty::new(value),
                                    span:  expression.span,
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

kinds!(
    Bool,   "bool",   bool;
    Number, "number", u32;
    List,   "list",   Vec<Expression>;
    String, "string", StdString;
    Word,   "word",   StdString;
);


impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Bool(b)        => b.fmt(f),
            Kind::Number(number) => number.fmt(f),
            Kind::List(list)     => fmt_list(list, f),
            Kind::String(string) => string.fmt(f),
            Kind::Word(word)     => word.fmt(f),
        }
    }
}


impl Not for Bool {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bool::new(self.inner.not())
    }
}


impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Number::new(self.inner + rhs.inner)
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Number::new(self.inner * rhs.inner)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}


impl IntoIterator for List {
    type Item     = <Vec<Expression> as IntoIterator>::Item;
    type IntoIter = <Vec<Expression> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_list(&self.inner, f)
    }
}


fn fmt_list(list: &Vec<Expression>, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[ ")?;
    for item in list {
        write!(f, "{} ", item.kind)?;
    }
    write!(f, "]")?;

    Ok(())
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
