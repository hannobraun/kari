use std::fmt;

use crate::{
    context::{
        self,
        Context,
    },
    data::{
        functions::Scope,
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

impl<H> fmt::Debug for Function<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Builtin(_)           => write!(f, "builtin"),
            Function::UserDefined { body } => write!(f, "{:?}", body),
        }
    }
}


pub type Builtin<Host> =
    fn(&mut Host, &mut dyn Context<Host>, Scope, Span)
        -> Result<(), context::Error>;
