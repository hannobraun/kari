use std::fmt;

use crate::data::{
    expr::Any,
    span::Span,
};


pub trait Type {
    const NAME: &'static str;

    type Value;

    fn from_any(_: Any) -> Result<Self::Value, Any>;

    fn check(any: Any) -> Result<Self::Value, TypeError>

    {
        Self::from_any(any)
            .map_err(|expression|
                TypeError {
                    expected: Self::NAME,
                    actual:   expression,
                }
            )
    }
}

impl Type for Any {
    const NAME: &'static str = "expression";

    type Value = Self;

    fn from_any(expression: Any) -> Result<Self::Value, Any> {
        Ok(expression)
    }
}


#[derive(Debug)]
pub struct TypeError {
    pub expected: &'static str,
    pub actual:   Any,
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
