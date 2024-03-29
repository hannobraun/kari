use crate::{functions::Scope, source::Span};

#[derive(Clone, Debug, Default)]
pub struct CallStack {
    pub frames: Vec<StackFrame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn operator(&self) -> &StackFrame {
        self.frames
            .last()
            // This shouldn't generally happen, as this is only called by
            // builtins, and when a builtin is executed, there must be a frame
            // on the call stack.
            .expect("Tried to get operator from empty call stack")
    }

    pub fn caller(&self) -> Option<&StackFrame> {
        if self.frames.len() < 2 {
            return None;
        }

        Some(&self.frames[self.frames.len() - 2])
    }
}

#[derive(Clone, Debug)]
pub struct StackFrame {
    pub scope: Scope,
    pub span: Option<Span>,
}
