use std::fmt;

use crate::data::{
    expr,
    span::Span,
};


pub trait Type {
    const NAME: &'static str;

    type Value: expr::Expr;

    fn from_any(&self, _: expr::Any) -> Result<Self::Value, expr::Any>;

    fn check(&self, any: expr::Any) -> Result<Self::Value, TypeError> {
        self.from_any(any)
            .map_err(|expression|
                TypeError {
                    expected: Self::NAME,
                    actual:   expression,
                }
            )
    }
}


pub struct Any;

impl Type for Any {
    const NAME: &'static str = "any";

    type Value = expr::Any;

    fn from_any(&self, any: expr::Any)
        -> Result<Self::Value, expr::Any>
    {
        Ok(any)
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

                fn from_any(&self, expression: expr::Any)
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
