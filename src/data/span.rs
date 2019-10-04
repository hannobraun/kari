use crate::data::ch::Position;


#[derive(Clone, Debug, Default)]
pub struct Span {
    /// The stream this span refers to
    pub stream: String,

    /// The position of the first character in the span
    pub start: Position,

    /// The position of the last character in the span
    pub end: Position,
}

impl Span {
    pub fn merge(mut self, other: &Self) -> Self {
        // The following code obviously assumes something like the this
        // assertion, but uncommenting the assertion will result in panics. This
        // has been documented in the BUGS file.
        // assert_eq!(self.stream, other.stream);

        if self.start > other.start {
            self.start = other.start;
        }
        if self.end < other.end {
            self.end = other.end;
        }
        self
    }
}
