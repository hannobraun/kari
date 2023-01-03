use decorum::R32;

use crate::source::{Span, Token, TokenKind};

pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Option<Span>,
}

pub enum ExpressionKind {
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
            TokenKind::Bool(value) => ExpressionKind::Bool(value),
            TokenKind::Float(value) => ExpressionKind::Float(value),
            TokenKind::Number(value) => ExpressionKind::Number(value),
            TokenKind::String(value) => ExpressionKind::String(value),
            TokenKind::Symbol(value) => ExpressionKind::Symbol(value),
            TokenKind::Word(value) => ExpressionKind::Word(value),

            kind => panic!("Can convert {} to value", kind),
        };

        Self {
            kind,
            span: token.span,
        }
    }
}
