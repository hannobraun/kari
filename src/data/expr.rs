use std::{
    fmt,
    mem::discriminant,
    string::String as StdString,
};

use crate::data::{
    span::Span,
    token::{
        self,
        Token,
    },
    types::Type,
};


pub trait Expr : Sized {
    type Inner;

    fn new(_: Self::Inner, _: Span) -> Self;
    fn open(self) -> (Self::Inner, Span);

    fn into_any(self) -> Any;
}


#[derive(Clone, Debug)]
pub struct Any {
    pub kind: Kind,
    pub span: Span,
}

impl Any {
    pub fn from_token(token: Token) -> Self {
        let kind = match token.kind {
            token::Kind::Bool(value)   => Kind::Bool(value),
            token::Kind::Float(value)  => Kind::Float(value),
            token::Kind::Number(value) => Kind::Number(value),
            token::Kind::String(value) => Kind::String(value),
            token::Kind::Symbol(value) => Kind::Symbol(value),
            token::Kind::Word(value)   => Kind::Word(value),

            kind => panic!("Can convert {} to expression", kind),
        };

        Self {
            kind,
            span: token.span,
        }
    }
}

impl Type for Any {
    const NAME: &'static str = "expression";

    type Value = Self;

    fn from_any(expression: Any) -> Result<Self::Value, Any> {
        Ok(expression)
    }
}

impl Expr for Any {
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

            impl Type for $ty {
                const NAME: &'static str = $name;

                type Value = Self;

                fn from_any(expression: Any)
                    -> Result<Self::Value, Any>
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

            impl Expr for $ty {
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
            }
        )*
    }
}

kinds!(
    Bool,   "bool",   bool;
    Float,  "float",  f32;
    Number, "number", u32;
    List,   "list",   Vec<Any>;
    String, "string", StdString;
    Symbol, "symbol", StdString;
    Word,   "word",   StdString;
);


impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        // Determines equality based on data, ignoring spans.
        match (self, other) {
            (Kind::Bool(a),   Kind::Bool(b))   => return a == b,
            (Kind::Number(a), Kind::Number(b)) => return a == b,
            (Kind::String(a), Kind::String(b)) => return a == b,
            (Kind::Symbol(a), Kind::Symbol(b)) => return a == b,
            (Kind::Word(a),   Kind::Word(b))   => return a == b,

            (Kind::List(a), Kind::List(b)) => {
                if a.len() != b.len() {
                    return false;
                }

                for (a, b) in a.iter().zip(b.iter()) {
                    if a.kind != b.kind {
                        return false;
                    }
                }

                true
            }

            _ => {
                // When this was written, all the same-variant combinations were
                // covered. But surely more variants will be added, making this
                // code incomplete.
                //
                // Panic, if we detect that both variants are the same, as that
                // means this code needs to be extended.
                assert_eq!(discriminant(self), discriminant(other));

                // If we haven't panicked by this point, we have two different
                // variants, which can't be equal.
                false
            }
        }
    }
}

impl Eq for Kind {}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Bool(value)   => value.fmt(f),
            Kind::Float(value)  => write!(f, "{:?}", value),
            Kind::Number(value) => value.fmt(f),
            Kind::List(value)   => fmt_list(value, f),
            Kind::String(value) => value.fmt(f),
            Kind::Symbol(value) => write!(f, ":{}", value),
            Kind::Word(value)   => value.fmt(f),
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
