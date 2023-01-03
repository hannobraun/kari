use decorum::R32;

use crate::source::{Span, Token, TokenKind};

pub struct Expression {
    pub kind: Kind,
    pub span: Option<Span>,
}

pub enum Kind {
    Bool(bool),
    Float(R32),
    Number(u32),
    List(Vec<Expression>),
    String(String),
    Symbol(String),
    Word(String),
}

impl Expression {
    pub fn from_token(token: Token) -> Self {
        let kind = match token.kind {
            TokenKind::Bool(value) => Kind::Bool(value),
            TokenKind::Float(value) => Kind::Float(value),
            TokenKind::Number(value) => Kind::Number(value),
            TokenKind::String(value) => Kind::String(value),
            TokenKind::Symbol(value) => Kind::Symbol(value),
            TokenKind::Word(value) => Kind::Word(value),

            kind => panic!("Can convert {} to value", kind),
        };

        Self {
            kind,
            span: token.span,
        }
    }
}
