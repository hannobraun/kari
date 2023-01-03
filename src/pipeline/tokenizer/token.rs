use std::fmt;

use decorum::R32;

use crate::source::Span;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Option<Span>,
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    ListOpen,
    ListClose,
    Bool(bool),
    Float(R32),
    Number(u32),
    String(String),
    Symbol(String),
    Word(String),
}

impl TokenKind {
    pub fn parse_word(word: String) -> Self {
        if let Ok(value) = word.parse::<bool>() {
            return TokenKind::Bool(value);
        }
        if let Ok(value) = word.parse::<u32>() {
            return TokenKind::Number(value);
        }
        if let Ok(value) = word.parse::<R32>() {
            return TokenKind::Float(value);
        }

        TokenKind::Word(word)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::ListOpen => write!(f, "["),
            TokenKind::ListClose => write!(f, "]"),
            TokenKind::Bool(value) => value.fmt(f),
            TokenKind::Float(value) => write!(f, "{:?}", value),
            TokenKind::Number(value) => value.fmt(f),
            TokenKind::String(value) => value.fmt(f),
            TokenKind::Symbol(value) => write!(f, ":{}", value),
            TokenKind::Word(value) => value.fmt(f),
        }
    }
}
