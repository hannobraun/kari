use crate::source::Position;

/// A location in the source code
///
/// Used to identify where tokens, values, etc. originate in the source code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    /// The stream this source refers to
    pub stream_name: String,

    /// The position in the stream of this source's first character
    pub start: Position,

    /// The position in the stream of this source's last character
    pub end: Position,
}

pub trait MergeSpans {
    fn merge(self, other: Self) -> Self;
}

impl MergeSpans for Option<Span> {
    fn merge(self, other: Self) -> Self {
        match self {
            None => other.clone(),
            Some(mut self_) => {
                match other {
                    None => Some(self_),
                    Some(other) => {
                        // The following code obviously assumes something like
                        // the this assertion, but uncommenting the assertion
                        // will result in panics. This has been documented in
                        // the BUGS file.
                        // assert_eq!(self.stream, other.stream);

                        if self_.start > other.start {
                            self_.start = other.start;
                        }
                        if self_.end < other.end {
                            self_.end = other.end;
                        }

                        Some(self_)
                    }
                }
            }
        }
    }
}
