#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    /// The line, starting at 0
    pub line: usize,

    /// The column, starting at 0
    pub column: usize,

    /// The byte index within the stream
    pub index: usize,
}
