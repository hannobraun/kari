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


pub type Builtin<Host> =
    fn(&mut Host, &mut dyn Context<Host>, Scope, Span)
        -> Result<(), context::Error>;
