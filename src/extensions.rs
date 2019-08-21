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


pub type Extension<Host> =
    fn(Rc<RefCell<Host>>, &mut dyn Context, Span) -> Result<(), context::Error>;


pub struct Extensions<Host>(HashMap<String, Extension<Host>>);

impl<Host> Extensions<Host> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, name: &str) -> Option<Extension<Host>> {
        self.0
            .get(name)
            .map(|extension| *extension)
    }
}

impl Extensions<()> {
    pub fn none() -> Self {
        Extensions::new()
    }
}
