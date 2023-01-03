mod char;
mod position;
mod span;

pub use self::{
    char::Char,
    position::Position,
    span::{MergeSpans, Span},
};
