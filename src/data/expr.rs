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

use crate::data::span::Span;


pub trait Expr : Sized {
    const NAME: &'static str;

    fn into_any(self) -> Any;
    fn from_expr(expression: Any) -> Result<Self, Any>;
}


#[derive(Clone, Debug)]
pub struct Any {
    pub kind: Kind,
    pub span: Span,
}

impl Any {
    pub fn check<T>(self) -> Result<T, Error>
        where
            T: Expr,
    {
        T::from_expr(self)
            .map_err(|expression|
                Error::TypeError {
                    expected: T::NAME,
                    actual:   expression,
                }
            )
    }
}

impl Expr for Any {
    const NAME: &'static str = "expression";

    fn into_any(self) -> Any {
        self
    }

    fn from_expr(expression: Any) -> Result<Self, Any> {
        Ok(expression)
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
                pub span:  Span,
            }

            impl $ty {
                pub fn new(inner: $inner, span: Span) -> Self {
                    Self {
                        inner,
                        span,
                    }
                }
            }

            impl Expr for $ty {
                const NAME: &'static str = $name;

                fn into_any(self) -> Any {
                    Any {
                        kind: Kind::$ty(self.inner),
                        span: self.span,
                    }
                }

                fn from_expr(expression: Any)
                    -> Result<Self, Any>
                {
                    match expression.kind {
                        Kind::$ty(value) => {
                            Ok($ty::new(value, expression.span))
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
    List,   "list",   Vec<Any>;
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
        Bool::new(
            self.inner.not(),
            self.span,
        )
    }
}


impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Number::new(
            self.inner + rhs.inner,
            self.span.merge(rhs.span),
        )
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Number::new(
            self.inner * rhs.inner,
            self.span.merge(rhs.span),
        )
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
    type Item     = <Vec<Any> as IntoIterator>::Item;
    type IntoIter = <Vec<Any> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_list(&self.inner, f)
    }
}


fn fmt_list(list: &Vec<Any>, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[ ")?;
    for item in list {
        write!(f, "{} ", item.kind)?;
    }
    write!(f, "]")?;

    Ok(())
}


#[derive(Debug)]
pub enum Error {
    TypeError {
        expected: &'static str,
        actual:   Any,
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
