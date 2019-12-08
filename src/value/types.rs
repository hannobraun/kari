use std::{
    fmt,
    hash::{
        Hash,
        Hasher,
    },
};

use crate::{
    pipeline::tokenizer::Source,
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


pub trait Downcast {
    type Output;

    fn downcast(&self, any: value::Any) -> Result<Self::Output, TypeError>;
}


#[derive(Debug)]
pub struct Any;

impl Type for Any {
    fn name(&self) -> &'static str { "any" }
}

impl Downcast for Any {
    type Output = value::Any;

    fn downcast(&self, any: value::Any) -> Result<Self::Output, TypeError> {
        Ok(any)
    }
}


#[derive(Debug, Eq, PartialEq)]
pub struct TypeError {
    pub expected: &'static str,
    pub actual:   value::Any,
}

impl TypeError {
    pub fn sources<'r>(&'r self, sources: &mut Vec<&'r Source>) {
        sources.push(&self.actual.src);
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
