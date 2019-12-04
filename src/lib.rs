pub mod builtins;
pub mod call_stack;
pub mod ch;
pub mod context;
pub mod expression;
pub mod functions;
pub mod interpreter;
pub mod pipeline;
pub mod stack;
pub mod token;
pub mod types;
pub mod value;


pub mod prelude {
    pub use crate::{
        types::Downcast as _,
        value::{
            Compute as _,
            Value as _,
        },
    };
}


pub use crate::{
    context::Context,
    interpreter::Interpreter,
};
