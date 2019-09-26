use std::{
    fmt,
    mem::discriminant,
    string::String as StdString,
};

use crate::data::{
    expression::{
        self,
        Expression,
    },
    functions::Scope as Scope_,
    span::Span,
};


pub trait Value : Sized {
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
    pub fn from_expression(expression: Expression, scope: Scope_) -> Self {
        let kind = match expression.kind {
            expression::Kind::Bool(inner)   => Kind::Bool(inner),
            expression::Kind::Float(inner)  => Kind::Float(inner),
            expression::Kind::Number(inner) => Kind::Number(inner),
            expression::Kind::String(inner) => Kind::String(inner),
            expression::Kind::Symbol(inner) => Kind::Symbol(inner),
            expression::Kind::Word(inner)   => Kind::Word(inner),

            expression::Kind::List(inner) => {
                Kind::List(ListInner::from_expressions(inner, scope))
            }
        };

        Self {
            kind,
            span: expression.span,
        }
    }
}

impl Value for Any {
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

            impl Value for $ty {
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
    Bool,   bool;
    Float,  f32;
    Number, u32;
    List,   ListInner;
    String, StdString;
    Symbol, StdString;
    Word,   StdString;
);


impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        // Determines equality based on data, ignoring spans.
        match (self, other) {
            (Kind::Bool(a),   Kind::Bool(b))   => return a == b,
            (Kind::Float(a),  Kind::Float(b))  => return a == b,
            (Kind::Number(a), Kind::Number(b)) => return a == b,
            (Kind::String(a), Kind::String(b)) => return a == b,
            (Kind::Symbol(a), Kind::Symbol(b)) => return a == b,
            (Kind::Word(a),   Kind::Word(b))   => return a == b,

            (Kind::List(a), Kind::List(b)) => {
                if a.items.len() != b.items.len() {
                    return false;
                }

                for (a, b) in a.items.iter().zip(b.items.iter()) {
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
                assert_ne!(discriminant(self), discriminant(other));

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
            Kind::List(value)   => fmt_list(&value.items, f),
            Kind::String(value) => value.fmt(f),
            Kind::Symbol(value) => write!(f, ":{}", value),
            Kind::Word(value)   => value.fmt(f),
        }
    }
}


#[derive(Clone, Debug)]
pub struct ListInner {
    pub items: Vec<Any>,
    pub scope: Scope_,
}

impl ListInner {
    pub fn from_expressions(expressions: Vec<Expression>, scope: Scope_)
        -> Self
    {
        let items = expressions
            .into_iter()
            .map(|e| Any::from_expression(e, scope))
            .collect();

        Self::from_values(items, scope)
    }

    pub fn from_values(values: Vec<Any>, scope: Scope_) -> Self {
        Self {
            items: values,
            scope,
        }
    }
}


impl IntoIterator for List {
    type Item     = <Vec<Any> as IntoIterator>::Item;
    type IntoIter = <Vec<Any> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.items.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_list(&self.inner.items, f)
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
            Out: Value<Inner=R>,
            F:   Fn(Self::In) -> R;
}

impl<T> Compute for T where T: Value {
    type In = T::Inner;

    fn compute<Out, F, R>(self, f: F) -> Out
        where
            Out: Value<Inner=R>,
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
        A: Value,
        B: Value,
{
    type In = (A::Inner, B::Inner);

    fn compute<Out, F, R>(self, f: F) -> Out
        where
            Out: Value<Inner=R>,
            F:   Fn(Self::In) -> R,
    {
        let (a_inner, a_span) = self.0.open();
        let (b_inner, b_span) = self.1.open();
        Out::new(
            f((a_inner, b_inner)),
            a_span.merge(&b_span),
        )
    }
}
