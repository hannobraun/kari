use crate::data::span::Span;


#[derive(Clone, Debug)]
pub struct CallStack {
    pub frames: Vec<StackFrame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }

    pub fn operator(&self) -> &StackFrame {
        self.frames.last()
            // This shouldn't generally happen, as this is only called by
            // builtins, and when a builtin is executed, there must be a frame
            // on the call stack.
            .expect("Tried to get operator from empty call stack")
    }
}


#[derive(Clone, Debug)]
pub struct StackFrame {
    pub span: Span,
}
