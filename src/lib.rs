pub mod builtins;
pub mod call_stack;
pub mod context;
pub mod functions;
pub mod interpreter;
pub mod pipeline;
pub mod stack;
pub mod value;


pub mod prelude {
    pub use crate::{
        pipeline::Stage as _,
        value::{
            Value as _,
            compute::Compute as _,
            types::Downcast as _,
        },
    };
}


pub use crate::{
    context::Context,
    interpreter::Interpreter,
};
