use std::{
    fmt,
    hash::{
        Hash,
        Hasher,
    },
};

use crate::data::{
    span::Span,
    value,
};


pub trait Typed {
    fn get_type(&self) -> &'static dyn Type;
}


pub trait Type : fmt::Debug {
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

impl Hash for dyn Type {
    fn hash<H>(&self, _: &mut H) where H: Hasher {
        // All types must have the same hash, because all types are equal to
        // `Any`. This meant we can't feed anything into the hasher here.
    }
}


pub trait Downcast : Type {
    type Value: value::Value;

    fn downcast_raw(&self, _: value::Any) -> Result<Self::Value, value::Any>;

    fn downcast(&self, any: value::Any) -> Result<Self::Value, TypeError> {
        self.downcast_raw(any)
            .map_err(|any|
                TypeError {
                    expected: self.name(),
                    actual:   any,
                }
            )
    }
}


#[derive(Debug)]
pub struct Any;

impl Type for Any {
    fn name(&self) -> &'static str { "any" }
}

impl Downcast for Any {
    type Value = value::Any;

    fn downcast_raw(&self, any: value::Any) -> Result<Self::Value, value::Any> {
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
                type Value = value::$ty;

                fn downcast_raw(&self, any: value::Any)
                    -> Result<Self::Value, value::Any>
                {
                    match any.kind {
                        value::Kind::$ty(value) => {
                            Ok(value::Value::new(value, any.span))
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
    pub actual:   value::Any,
}

impl TypeError {
    pub fn spans<'r>(&'r self, spans: &mut Vec<&'r Span>) {
        spans.push(&self.actual.span);
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
