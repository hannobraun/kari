use std::fmt;

use crate::data::span::Span;


#[derive(Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}


#[derive(Clone, Debug)]
pub enum Kind {
    Number(u32),
    ListOpen,
    ListClose,
    String(String),
    Symbol(String),
    Word(String),
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Number(number) => number.fmt(f),
            Kind::ListOpen       => write!(f, "["),
            Kind::ListClose      => write!(f, "]"),
            Kind::String(string) => string.fmt(f),
            Kind::Symbol(symbol) => write!(f, ":{}", symbol),
            Kind::Word(word)     => word.fmt(f),
        }
    }
}
