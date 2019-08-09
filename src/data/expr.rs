use std::{
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
            $inner:ty,
            derive($(
                $trait:ident,
            )*);
        )*
    ) => {
        #[derive(Clone, Debug)]
        pub enum Kind {
            $($ty($inner),)*
        }


        $(
            #[derive($($trait,)* Clone, Debug)]
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
    Bool,   "bool",   bool,            derive();
    Number, "number", u32,             derive(Eq, Ord, PartialEq, PartialOrd,);
    List,   "list",   Vec<Expression>, derive();
    String, "string", StdString,       derive();
    Word,   "word",   StdString,       derive();
);


impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Bool(b)        => b.fmt(f),
            Kind::Number(number) => number.fmt(f),
            Kind::List(list)     => List::new(list.clone()).fmt(f),
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


impl IntoIterator for List {
    type Item     = <Vec<Expression> as IntoIterator>::Item;
    type IntoIter = <Vec<Expression> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for item in &self.inner {
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
