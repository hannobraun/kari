use super::Position;


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
