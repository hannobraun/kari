use crate::pipeline::tokenizer::{
    Source,
    Token,
    token,
};


pub struct Expression {
    pub kind: Kind,
    pub span: Source,
}


pub enum Kind {
    Bool(bool),
    Float(f32),
    Number(u32),
    List(Vec<Expression>),
    String(String),
    Symbol(String),
    Word(String),
}

impl Expression {
    pub fn from_token(token: Token) -> Self {
        let kind = match token.kind {
            token::Kind::Bool(value)   => Kind::Bool(value),
            token::Kind::Float(value)  => Kind::Float(value),
            token::Kind::Number(value) => Kind::Number(value),
            token::Kind::String(value) => Kind::String(value),
            token::Kind::Symbol(value) => Kind::Symbol(value),
            token::Kind::Word(value)   => Kind::Word(value),

            kind => panic!("Can convert {} to value", kind),
        };

        Self {
            kind,
            span: token.span,
        }
    }
}
