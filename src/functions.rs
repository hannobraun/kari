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
};

pub struct Functions<T: Copy> {
    functions: HashMap<Signature, T>,
}

impl<T> Functions<T> where T: Copy {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn with(&mut self, name: String, function: T) -> &mut Self {
        self.functions.insert(Signature { name }, function);
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
