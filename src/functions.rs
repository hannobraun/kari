use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    hash::{
        Hash,
        Hasher,
    },
    rc::Rc,
};

use crate::{
    context::{
        self,
        Context,
    },
    data::{
        span::Span,
        stack::Stack,
        types::{
            Type,
            Typed,
        },
    },
};

pub struct Functions<T> {
    functions:  HashMap<Signature, T>,
    signatures: HashMap<String, usize>,
}

impl<T> Functions<T> where T: Clone {
    pub fn new() -> Self {
        Self {
            functions:  HashMap::new(),
            signatures: HashMap::new(),
        }
    }

    pub fn define(&mut self,
        name:     String,
        args:     &[&'static dyn Type],
        function: T,
    )
        -> Result<&mut Self, Error>
    {
        self.functions.insert(
            Signature {
                name: name.clone(),
                args: args.to_vec(),
            },
            function,
        );

        self.signatures
            .entry(name)
            .and_modify(|num| *num = usize::max(*num, args.len()))
            .or_insert(args.len());

        Ok(self)
    }

    pub fn get(&self, name: &str, stack: &Stack) -> Option<T> {
        for n in 0 ..= self.signatures.get(name).map(|n| *n).unwrap_or(0) {
            let mut args: Vec<&'static dyn Type> = stack
                .peek()
                .take(n)
                .map(|expr| expr.get_type())
                .collect();
            args.reverse();

            let function = self.functions
                .get(&Signature { name: String::from(name), args })
                .map(|function| function.clone());

            if function.is_some() {
                return function;
            }
        }

        None
    }
}


pub struct Signature {
    pub name: String,
    pub args: Vec<&'static dyn Type>,
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        let args_are_equal = self.args
            .iter()
            .zip(other.args.iter())
            .fold(true, |p, (&a1, &a2)| p && a1.eq(a2));

        self.name == other.name
            && args_are_equal
    }
}

impl Eq for Signature {}

impl Hash for Signature {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.name.hash(state);
        // Arguments can't be part of hash, as types can have different names,
        // but still be equal (when one of them is "any").
    }
}


#[derive(Debug)]
pub enum Error {
    FunctionAlreadyDefined,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FunctionAlreadyDefined =>
                write!(f, "Function already defined"),
        }
    }
}


pub type Builtin =
    fn(&mut dyn Context, Span) -> Result<(), context::Error>;
pub type Extension<Host> =
    fn(Rc<RefCell<Host>>, &mut dyn Context, Span) -> Result<(), context::Error>;
