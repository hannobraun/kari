use std::fmt;

use crate::{
    stream::Stream,
    tokenizer::{
        self,
        Span,
        Token,
        TokenKind,
    },
};


pub struct Parser<Tokenizer> {
    tokenizer: Tokenizer,
}

impl<Tokenizer> Parser<Tokenizer> {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Parser {
            tokenizer,
        }
    }
}

impl<Tokenizer> Stream for Parser<Tokenizer>
    where Tokenizer: Stream<Item=Token, Error=tokenizer::Error>
{
    type Item  = Expression;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        let token = self.tokenizer.next()?;

        let data = match token.kind {
            TokenKind::ListOpen => {
                ExpressionKind::List(self.parse_list()?)
            }
            TokenKind::ListClose => {
                return Err(Error::UnexpectedToken(token));
            }
            TokenKind::Number(number) => {
                ExpressionKind::Number(Number(number))
            }
            TokenKind::String(string) => {
                ExpressionKind::String(string)
            }
            TokenKind::Word(word) => {
                ExpressionKind::Word(word)
            }
        };

        Ok(
            Expression {
                data,
            }
        )
    }
}

impl<Tokenizer> Parser<Tokenizer>
    where Tokenizer: Stream<Item=Token, Error=tokenizer::Error>
{
    fn parse_list(&mut self) -> Result<List, Error> {
        let mut list = List::new();

        loop {
            let token = self.tokenizer.next()?;

            let data = match token.kind {
                TokenKind::ListOpen => {
                    ExpressionKind::List(self.parse_list()?)
                }
                TokenKind::ListClose => {
                    return Ok(list);
                }
                TokenKind::Number(number) => {
                    ExpressionKind::Number(Number(number))
                }
                TokenKind::String(string) => {
                    ExpressionKind::String(string)
                }
                TokenKind::Word(word) => {
                    ExpressionKind::Word(word)
                }
            };

            list.0.push(
                Expression {
                    data,
                }
            );
        }
    }
}


#[derive(Clone, Debug)]
pub struct Expression {
    pub data: ExpressionKind,
}


#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Bool(Bool),
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl fmt::Display for ExpressionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpressionKind::Bool(b)        => b.0.fmt(f),
            ExpressionKind::Number(number) => number.0.fmt(f),
            ExpressionKind::List(list)     => list.fmt(f),
            ExpressionKind::String(string) => string.fmt(f),
            ExpressionKind::Word(word)     => word.fmt(f),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Bool(pub bool);


#[derive(Clone, Debug)]
pub struct Number(pub u32);


#[derive(Clone, Debug)]
pub struct List(pub Vec<Expression>);

impl List {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl IntoIterator for List {
    type Item     = <Vec<Expression> as IntoIterator>::Item;
    type IntoIter = <Vec<Expression> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for item in &self.0 {
            write!(f, "{} ", item.data)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}


pub trait ToExpression {
    fn to_expression(self) -> Expression;
}

impl ToExpression for Expression {
    fn to_expression(self) -> Expression {
        self
    }
}

macro_rules! impl_expression {
    ($($name:ident;)*) => {
        $(
            impl ToExpression for $name {
                fn to_expression(self) -> Expression {
                    Expression {
                        data: ExpressionKind::$name(self),
                    }
                }
            }
        )*
    }
}

impl_expression!(
    Bool;
    Number;
    List;
    String;
);


#[derive(Debug)]
pub enum Error {
    Tokenizer(tokenizer::Error),
    UnexpectedToken(Token),
    EndOfStream,
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::Tokenizer(_)           => None,
            Error::UnexpectedToken(token) => Some(token.span),
            Error::EndOfStream            => None,
        }
    }
}

impl From<tokenizer::Error> for Error {
    fn from(from: tokenizer::Error) -> Self {
        match from {
            tokenizer::Error::EndOfStream => Error::EndOfStream,
            error                         => Error::Tokenizer(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Tokenizer(error) => {
                write!(f, "Tokenizer error:\n{:?}", error)?;
            }
            Error::UnexpectedToken(token) => {
                write!(f, "Unexpected token: `{}`", token.kind)?;
            }
            Error::EndOfStream => {
                panic!("Error variant should not be display: {:?}", self);
            }
        }

        Ok(())
    }
}
