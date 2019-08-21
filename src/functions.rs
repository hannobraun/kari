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

pub struct Functions<T: Copy>(HashMap<Signature, T>);

impl<T> Functions<T> where T: Copy {
    pub fn new(functions: HashMap<Signature, T>) -> Self {
        Self(functions)
    }

    pub fn none() -> Self {
        Self::new(HashMap::new())
    }

    pub fn get(&self, name: &str) -> Option<T> {
        self.0
            .get(name)
            .map(|function| *function)
    }
}


pub type Signature = String;


pub type Builtin =
    fn(&mut dyn Context, Span) -> Result<(), context::Error>;
pub type Extension<Host> =
    fn(Rc<RefCell<Host>>, &mut dyn Context, Span) -> Result<(), context::Error>;
