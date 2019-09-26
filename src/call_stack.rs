use crate::data::span::Span;


pub type CallStack = Vec<StackFrame>;

#[derive(Clone, Debug)]
pub struct StackFrame {
    pub span: Span,
}
