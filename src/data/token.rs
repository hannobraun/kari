use std::fmt;

use crate::data::span::Span;


#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}


#[derive(Clone, Debug)]
pub enum TokenKind {
    Number(u32),
    ListOpen,
    ListClose,
    String(String),
    Symbol(String),
    Word(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Number(number) => number.fmt(f),
            TokenKind::ListOpen       => write!(f, "["),
            TokenKind::ListClose      => write!(f, "]"),
            TokenKind::String(string) => string.fmt(f),
            TokenKind::Symbol(symbol) => write!(f, ":{}", symbol),
            TokenKind::Word(word)     => word.fmt(f),
        }
    }
}
