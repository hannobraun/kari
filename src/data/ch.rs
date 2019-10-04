#[derive(Clone, Copy, Debug)]
pub struct Char {
    pub c:   char,
    pub pos: Position,
}

impl Char {
    pub fn is_whitespace(&self) -> bool {
        self.c.is_whitespace()
    }
}

impl PartialEq<char> for Char {
    fn eq(&self, other: &char) -> bool {
        self.c.eq(other)
    }
}



#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    /// The line, starting at 0
    pub line: usize,

    /// The column, starting at 0
    pub column: usize,

    /// The byte index within the stream
    pub index: usize,
}
