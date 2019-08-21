use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::{
    context::{
        self,
        Context,
    },
    data::span::Span,
    data::types::Type,
};

pub struct Functions<T: Copy> {
    functions:  HashMap<Signature, T>,
    signatures: HashMap<String, usize>,
}

impl<T> Functions<T> where T: Copy {
    pub fn new() -> Self {
        Self {
            functions:  HashMap::new(),
            signatures: HashMap::new(),
        }
    }

    pub fn with(&mut self,
        name:     String,
        args:     Vec<&'static dyn Type>,
        function: T,
    )
        -> &mut Self
    {
        let args_len = args.len();
        self.functions.insert(Signature { name: name.clone() }, function);

        self.signatures
            .entry(name)
            .and_modify(|num| *num = usize::max(*num, args_len))
            .or_insert(args_len);

        self
    }

    pub fn get(&self, name: &str) -> Option<T> {
        self.functions
            .get(&Signature { name: String::from(name) })
            .map(|function| *function)
    }
}


#[derive(Eq, Hash, PartialEq)]
pub struct Signature {
    pub name: String,
}


pub type Builtin =
    fn(&mut dyn Context, Span) -> Result<(), context::Error>;
pub type Extension<Host> =
    fn(Rc<RefCell<Host>>, &mut dyn Context, Span) -> Result<(), context::Error>;
