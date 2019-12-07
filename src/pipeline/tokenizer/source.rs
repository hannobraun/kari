use crate::pipeline::reader::Position;


/// A location in the source code
///
/// Used to identify where tokens, values, etc. originate in the source code.
#[derive(Clone, Debug, Default)]
pub struct Source {
    /// The stream this source refers to
    pub stream: String,

    /// The position in the stream of this source's first character
    pub start: Position,

    /// The position in the stream of this source's last character
    pub end: Position,
}

impl Source {
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
