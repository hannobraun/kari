pub mod builtins;
pub mod call_stack;
pub mod context;
pub mod functions;
pub mod interpreter;
pub mod pipeline;
pub mod source;
pub mod stack;
pub mod value;

pub mod prelude {
    pub use crate::value::{
        cast::{Cast, Downcast as _},
        compute::Compute as _,
        Value as _,
    };
}

pub use crate::{context::Context, interpreter::Interpreter};
