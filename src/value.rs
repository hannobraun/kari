pub mod cast;
pub mod compute;
pub mod types;

use std::{fmt, mem::discriminant, string::String as String_};

use decorum::R32;

use crate::{
    functions::Scope as Scope_,
    source::{Expression, ExpressionKind, Span},
};

pub trait Value: Sized {
    type Inner;

    fn new(_: Self::Inner, _: Option<Span>) -> Self;
    fn open(self) -> (Self::Inner, Option<Span>);

    fn into_any(self) -> Any;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Any {
    pub kind: Kind,
    pub span: Option<Span>,
}

impl Any {
    pub fn from_expression(expression: Expression, scope: Scope_) -> Self {
        let kind = match expression.kind {
            ExpressionKind::Bool(inner) => Kind::Bool(inner),
            ExpressionKind::Float(inner) => Kind::Float(inner),
            ExpressionKind::Number(inner) => Kind::Number(inner),
            ExpressionKind::String(inner) => Kind::String(inner),
            ExpressionKind::Symbol(inner) => Kind::Symbol(inner),
            ExpressionKind::Word(inner) => Kind::Word(inner),

            ExpressionKind::List(inner) => {
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

    fn new(kind: Self::Inner, span: Option<Span>) -> Self {
        Self { kind, span }
    }

    fn open(self) -> (Self::Inner, Option<Span>) {
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


        pub mod v {
            use std::string::String as String_;

            use decorum::R32;

            use crate::{
                functions::Scope as Scope_,
                source::Span,
            };

            use super::{
                Kind,
                ListInner,
                Value,
            };


            pub use crate::value::Any;


            $(
                #[derive(Clone, Debug, Eq, PartialEq)]
                pub struct $ty {
                    pub inner: $inner,
                    pub span:  Option<Span>,
                }

                impl Value for $ty {
                    type Inner = $inner;

                    fn new(inner: $inner, span: Option<Span>) -> Self {
                        Self {
                            inner,
                            span,
                        }
                    }

                    fn open(self) -> (Self::Inner, Option<Span>) {
                        (self.inner, self.span)
                    }

                    fn into_any(self) -> Any {
                        Any {
                            kind: Kind::$ty(self.inner),
                            span: self.span,
                        }
                    }
                }

                impl From<$inner> for $ty {
                    fn from(inner: $inner) -> Self {
                        $ty {
                            inner,
                            span: None,
                        }
                    }
                }


                impl From<$ty> for Any {
                    fn from(ty: $ty) -> Self {
                        Any {
                            kind: Kind::$ty(ty.inner),
                            span: ty.span,
                        }
                    }
                }
            )*
        }

        pub mod t {
            use crate::{
                value::{
                    self,
                    Value,
                    cast::{
                        Downcast,
                        TypeError,
                    },
                    types::{
                        self,
                        Type,
                        Typed,
                    },
                    v,
                },
            };


            pub use types::Any;


            impl Typed for value::Any {
                fn get_type(&self) -> &'static dyn Type {
                    match self.kind {
                        $(value::Kind::$ty(_) => &$ty,)*
                    }
                }
            }


            $(
                #[derive(Debug)]
                pub struct $ty;

                impl Type for $ty {
                    fn name(&self) -> &'static str { $name }
                }

                impl Downcast for $ty {
                    type Input  = value::Any;
                    type Output = v::$ty;

                    fn downcast(&self, any: value::Any)
                        -> Result<Self::Output, TypeError>
                    {
                        match any.kind {
                            value::Kind::$ty(value) => {
                                Ok(Value::new(value, any.span))
                            }
                            _ => {
                                Err(
                                    TypeError {
                                        expected: self.name(),
                                        actual:   any,
                                    }
                                )
                            }
                        }
                    }
                }
            )*
        }
    }
}

kinds!(
    Bool,   "bool",   bool;
    Float,  "float",  R32;
    Number, "number", u32;
    List,   "list",   ListInner;
    Scope,  "scope",  Scope_;
    String, "string", String_;
    Symbol, "symbol", String_;
    Word,   "word",   String_;
);

impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        // Determines equality based on data, ignoring spans.
        match (self, other) {
            (Kind::Bool(a), Kind::Bool(b)) => a == b,
            (Kind::Float(a), Kind::Float(b)) => a == b,
            (Kind::Number(a), Kind::Number(b)) => a == b,
            (Kind::String(a), Kind::String(b)) => a == b,
            (Kind::Symbol(a), Kind::Symbol(b)) => a == b,
            (Kind::Word(a), Kind::Word(b)) => a == b,

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
            Kind::Bool(value) => value.fmt(f),
            Kind::Float(value) => write!(f, "{:?}", value),
            Kind::Number(value) => value.fmt(f),
            Kind::List(value) => fmt_list(&value.items, f),
            Kind::Scope(value) => write!(f, "{:?}", value),
            Kind::String(value) => value.fmt(f),
            Kind::Symbol(value) => write!(f, ":{}", value),
            Kind::Word(value) => value.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListInner {
    pub items: Vec<Any>,
    pub scope: Scope_,
}

impl ListInner {
    pub fn from_expressions(
        expressions: Vec<Expression>,
        scope: Scope_,
    ) -> Self {
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

impl IntoIterator for v::List {
    type Item = <Vec<Any> as IntoIterator>::Item;
    type IntoIter = <Vec<Any> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.items.into_iter()
    }
}

impl fmt::Display for v::List {
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
