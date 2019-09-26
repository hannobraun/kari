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
}


#[derive(Clone, Debug)]
pub struct StackFrame {
    pub span: Span,
}
