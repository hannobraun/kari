use std::fmt;

use super::Source;


#[derive(Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Source,
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
