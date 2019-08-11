use std::fmt;

use crate::data::span::Span;


#[derive(Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}


#[derive(Clone, Debug)]
pub enum Kind {
    ListOpen,
    ListClose,
    Number(u32),
    String(String),
    Symbol(String),
    Word(String),
}

impl Kind {
    pub fn parse_word(word: String) -> Self {
        if let Ok(number) = word.parse::<u32>() {
            return Kind::Number(number);
        }

        Kind::Word(word)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::ListOpen       => write!(f, "["),
            Kind::ListClose      => write!(f, "]"),
            Kind::Number(number) => number.fmt(f),
            Kind::String(string) => string.fmt(f),
            Kind::Symbol(symbol) => write!(f, ":{}", symbol),
            Kind::Word(word)     => word.fmt(f),
        }
    }
}
