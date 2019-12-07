use crate::pipeline::reader::Position;


/// A location in the source code
///
/// Used to identify where tokens, values, etc. originate in the source code.
#[derive(Clone, Debug)]
pub enum Source {
    /// A source that consists of a single region in a single file
    Continuous(Continuous),
}

impl Source {
    pub fn merge(self, other: &Self) -> Self {
        match self {
            Source::Continuous(mut self_) => {
                match other {
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

                        Source::Continuous(self_)
                    }
                }
            }
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Source::Continuous(Continuous::default())
    }
}


/// A source that consists of a single region in a single file
#[derive(Clone, Debug, Default)]
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
