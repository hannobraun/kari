use std::{
    cell::RefCell,
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
