use std::fmt;

use crate::ch::Position;


#[derive(Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}


#[derive(Clone, Debug)]
pub enum Kind {
    ListOpen,
    ListClose,
    Bool(bool),
    Float(f32),
    Number(u32),
    String(String),
    Symbol(String),
    Word(String),
}

impl Kind {
    pub fn parse_word(word: String) -> Self {
        if let Ok(value) = word.parse::<bool>() {
            return Kind::Bool(value);
        }
        if let Ok(value) = word.parse::<u32>() {
            return Kind::Number(value);
        }
        if let Ok(value) = word.parse::<f32>() {
            return Kind::Float(value)
        }

        Kind::Word(word)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::ListOpen      => write!(f, "["),
            Kind::ListClose     => write!(f, "]"),
            Kind::Bool(value)   => value.fmt(f),
            Kind::Float(value)  => write!(f, "{:?}", value),
            Kind::Number(value) => value.fmt(f),
            Kind::String(value) => value.fmt(f),
            Kind::Symbol(value) => write!(f, ":{}", value),
            Kind::Word(value)   => value.fmt(f),
        }
    }
}


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
