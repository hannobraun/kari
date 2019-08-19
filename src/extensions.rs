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


pub struct Extensions<Host> {
    pub host: Rc<RefCell<Host>>,
    pub map:  HashMap<String, Extension<Host>>,
}

impl<Host> Extensions<Host> {
    pub fn new(host: Host) -> Self {
        Self {
            host: Rc::new(RefCell::new(host)),
            map:  HashMap::new(),
        }
    }
}

impl Extensions<()> {
    pub fn none() -> Self {
        Extensions::new(())
    }
}
