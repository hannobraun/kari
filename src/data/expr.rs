use std::{
    fmt,
    string::String as StdString,
};

use crate::data::span::Span;


pub trait Expr : Sized {
    const NAME: &'static str;

    type Inner;

    fn new(_: Self::Inner, _: Span) -> Self;
    fn open(self) -> (Self::Inner, Span);

    fn into_any(self) -> Any;
    fn from_any(_: Any) -> Result<Self, Any>;
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
        T::from_any(self)
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

    type Inner = Kind;

    fn new(kind: Self::Inner, span: Span) -> Self {
        Self {
            kind,
            span,
        }
    }

    fn open(self) -> (Self::Inner, Span) {
        (self.kind, self.span)
    }

    fn into_any(self) -> Any {
        self
    }

    fn from_any(expression: Any) -> Result<Self, Any> {
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

            impl Expr for $ty {
                const NAME: &'static str = $name;

                type Inner = $inner;

                fn new(inner: $inner, span: Span) -> Self {
                    Self {
                        inner,
                        span,
                    }
                }

                fn open(self) -> (Self::Inner, Span) {
                    (self.inner, self.span)
                }

                fn into_any(self) -> Any {
                    Any {
                        kind: Kind::$ty(self.inner),
                        span: self.span,
                    }
                }

                fn from_any(expression: Any)
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


pub trait Compute {
    type In;

    fn compute<Out, F, R>(self, f: F) -> Out
        where
            Out: Expr<Inner=R>,
            F:   Fn(Self::In) -> R;
}

impl<T> Compute for T where T: Expr {
    type In = T::Inner;

    fn compute<Out, F, R>(self, f: F) -> Out
        where
            Out: Expr<Inner=R>,
            F:   Fn(Self::In) -> R,
    {
        let (inner, span) = self.open();
        Out::new(
            f(inner),
            span,
        )
    }
}

impl<A, B> Compute for (A, B)
    where
        A: Expr,
        B: Expr,
{
    type In = (A::Inner, B::Inner);

    fn compute<Out, F, R>(self, f: F) -> Out
        where
            Out: Expr<Inner=R>,
            F:   Fn(Self::In) -> R,
    {
        let (a_inner, a_span) = self.0.open();
        let (b_inner, b_span) = self.1.open();
        Out::new(
            f((a_inner, b_inner)),
            a_span.merge(b_span),
        )
    }
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
