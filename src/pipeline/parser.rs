use std::fmt;

use crate::{
    data::{
        expr::{
            self,
            Expression,
        },
        span::Span,
        token::{
            Token,
            TokenKind,
        },
    },
    pipeline::{
        self,
        tokenizer,
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

impl<Tokenizer> pipeline::Stage for Parser<Tokenizer>
    where Tokenizer: pipeline::Stage<Item=Token, Error=tokenizer::Error>
{
    type Item  = Expression;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        let token = self.tokenizer.next()?;

        let kind = match token.kind {
            TokenKind::ListOpen => {
                expr::Kind::List(self.parse_list()?)
            }
            TokenKind::ListClose => {
                return Err(Error::UnexpectedToken(token));
            }
            TokenKind::Number(number) => {
                expr::Kind::Number(expr::Number(number))
            }
            TokenKind::String(string) => {
                expr::Kind::String(expr::String(string))
            }
            TokenKind::Word(word) => {
                expr::Kind::Word(expr::Word(word))
            }
        };

        Ok(
            Expression {
                kind,
                span: token.span,
            }
        )
    }
}

impl<Tokenizer> Parser<Tokenizer>
    where Tokenizer: pipeline::Stage<Item=Token, Error=tokenizer::Error>
{
    fn parse_list(&mut self) -> Result<expr::List, Error> {
        let mut list = expr::List::new();

        loop {
            let token = self.tokenizer.next()?;

            let kind = match token.kind {
                TokenKind::ListOpen => {
                    expr::Kind::List(self.parse_list()?)
                }
                TokenKind::ListClose => {
                    return Ok(list);
                }
                TokenKind::Number(number) => {
                    expr::Kind::Number(expr::Number(number))
                }
                TokenKind::String(string) => {
                    expr::Kind::String(expr::String(string))
                }
                TokenKind::Word(word) => {
                    expr::Kind::Word(expr::Word(word))
                }
            };

            list.0.push(
                Expression {
                    kind,
                    span: token.span,
                }
            );
        }
    }
}


#[derive(Debug)]
pub enum Error {
    Tokenizer(tokenizer::Error),
    UnexpectedToken(Token),
    EndOfStream,
}

impl Error {
    pub fn span(self) -> Option<Span> {
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
