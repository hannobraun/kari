#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: Position,
    pub end:   Position,
}

impl Span {
    pub fn merge(mut self, other: Self) -> Self {
        if self.start > other.start {
            self.start = other.start;
        }
        if self.end < other.end {
            self.end = other.end;
        }
        self
    }
}


#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub line:   usize,
    pub column: usize,
}
