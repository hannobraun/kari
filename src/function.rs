use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    context::{
        self,
        Context,
    },
    data::{
        span::Span,
        value,
    },
};


pub enum Function<H> {
    Builtin(Builtin<H>),
    UserDefined {
        body: value::List,
    }
}

impl<H> Clone for Function<H> {
    fn clone(&self) -> Self {
        match self {
            Function::Builtin(f) => {
                Function::Builtin(f.clone())
            }
            Function::UserDefined { body } => {
                Function::UserDefined {
                    body: body.clone(),
                }
            }
        }
    }
}


pub type Host<H> = Rc<RefCell<H>>;

pub type Builtin<H> =
    fn(Host<H>, &mut dyn Context<H>, Span) -> Result<(), context::Error>;
