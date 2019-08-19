use std::fmt;

use crate::data::{
    expr,
    span::Span,
};


pub trait Type {
    const NAME: &'static str;

    type Value: expr::Expr;

    fn from_any(_: expr::Any) -> Result<Self::Value, expr::Any>;

    fn check(any: expr::Any) -> Result<Self::Value, TypeError> {
        Self::from_any(any)
            .map_err(|expression|
                TypeError {
                    expected: Self::NAME,
                    actual:   expression,
                }
            )
    }
}

impl Type for expr::Any {
    const NAME: &'static str = "expression";

    type Value = Self;

    fn from_any(expression: expr::Any) -> Result<Self::Value, expr::Any> {
        Ok(expression)
    }
}


macro_rules! impl_type {
    (
        $(
            $ty:ident,
            $name:expr;
        )*
    )
        =>
    {
        $(
            pub struct $ty;

            impl Type for $ty {
                const NAME: &'static str = $name;

                type Value = expr::$ty;

                fn from_any(expression: expr::Any)
                    -> Result<Self::Value, expr::Any>
                {
                    match expression.kind {
                        expr::Kind::$ty(value) => {
                            Ok(expr::Expr::new(value, expression.span))
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

impl_type!(
    Bool,   "bool";
    Float,  "float";
    Number, "number";
    List,   "list";
    String, "string";
    Symbol, "symbol";
    Word,   "word";
);


#[derive(Debug)]
pub struct TypeError {
    pub expected: &'static str,
    pub actual:   expr::Any,
}

impl TypeError {
    pub fn spans(self, spans: &mut Vec<Span>) {
        spans.push(self.actual.span);
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Type error: Expected `{}`, found `{}`",
            self.expected,
            self.actual.kind,
        )
    }
}
