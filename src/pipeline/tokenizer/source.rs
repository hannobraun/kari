use crate::pipeline::reader::Position;

/// A location in the source code
///
/// Used to identify where tokens, values, etc. originate in the source code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    /// Not a source
    ///
    /// Can be used in place of a source, where one would be expected, but none
    /// is available. This is the case in unit tests, for example.
    Null,

    /// A source that consists of a single region in a single file
    Continuous(Continuous),
}

impl Source {
    pub fn merge(self, other: Option<Self>) -> Option<Self> {
        match self {
            Source::Null => other,
            Source::Continuous(mut self_) => {
                match other.unwrap_or(Source::Null) {
                    Source::Null => Some(Source::Continuous(self_)),
                    Source::Continuous(other) => {
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

                        Some(Source::Continuous(self_))
                    }
                }
            }
        }
    }
}

/// A source that consists of a single region in a single file
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Continuous {
    /// The stream this source refers to
    pub stream: String,

    /// The position in the stream of this source's first character
    pub start: Position,

    /// The position in the stream of this source's last character
    pub end: Position,
}

impl Continuous {
    pub fn into_source(self) -> Source {
        Source::Continuous(self)
    }
}
