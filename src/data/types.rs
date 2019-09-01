use std::fmt;

use crate::data::{
    expr,
    span::Span,
};


pub trait Typed {
    fn get_type(&self) -> &'static dyn Type;
}


pub trait Type {
    fn name(&self) -> &'static str;
}

impl PartialEq for dyn Type {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            || self.name() == Any.name()
            || other.name() == Any.name()
    }
}

impl Eq for dyn Type {}


pub trait Downcast : Type {
    type Value: expr::Expr;

    fn downcast_raw(&self, _: expr::Any) -> Result<Self::Value, expr::Any>;

    fn downcast(&self, any: expr::Any) -> Result<Self::Value, TypeError> {
        self.downcast_raw(any)
            .map_err(|any|
                TypeError {
                    expected: self.name(),
                    actual:   any,
                }
            )
    }
}


pub struct Any;

impl Type for Any {
    fn name(&self) -> &'static str { "any" }
}

impl Downcast for Any {
    type Value = expr::Any;

    fn downcast_raw(&self, any: expr::Any) -> Result<Self::Value, expr::Any> {
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
        impl Typed for expr::Any {
            fn get_type(&self) -> &'static dyn Type {
                match self.kind {
                    $(expr::Kind::$ty(_) => &$ty,)*
                }
            }
        }

        $(
            pub struct $ty;

            impl Type for $ty {
                fn name(&self) -> &'static str { $name }
            }

            impl Downcast for $ty {
                type Value = expr::$ty;

                fn downcast_raw(&self, any: expr::Any)
                    -> Result<Self::Value, expr::Any>
                {
                    match any.kind {
                        expr::Kind::$ty(value) => {
                            Ok(expr::Expr::new(value, any.span))
                        }
                        _ => {
                            Err(any)
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
