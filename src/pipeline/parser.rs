pub mod expression;

pub use self::expression::Expression;

use std::{fmt, io};

use crate::pipeline::{
    self,
    tokenizer::{self, token, Source, Token},
};

use super::{tokenizer::source::Merge, Tokenizer};

pub struct Parser<R> {
    tokenizer: Tokenizer<R>,
}

impl<R> Parser<R> {
    pub fn new(tokenizer: Tokenizer<R>) -> Self {
        Parser { tokenizer }
    }
}

impl<R> pipeline::Stage for Parser<R>
where
    R: io::Read,
{
    type Item = Expression;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        let token = self.tokenizer.next()?;

        let expr = match token.kind {
            token::Kind::ListOpen => self.parse_list(token.src)?,
            token::Kind::ListClose => {
                return Err(Error::UnexpectedToken(token));
            }
            _ => Expression::from_token(token),
        };

        Ok(expr)
    }
}

impl<R> Parser<R>
where
    R: io::Read,
{
    fn parse_list(
        &mut self,
        mut list_source: Option<Source>,
    ) -> Result<Expression, Error> {
        let mut expressions = Vec::new();

        loop {
            let token = self.tokenizer.next()?;

            list_source = list_source.merge(token.src.clone());

            let expr = match token.kind {
                token::Kind::ListOpen => self.parse_list(token.src)?,
                token::Kind::ListClose => {
                    return Ok(Expression {
                        kind: expression::Kind::List(expressions),
                        src: list_source,
                    });
                }
                _ => Expression::from_token(token),
            };

            expressions.push(expr);
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
    pub fn sources<'r>(&'r self, sources: &mut Vec<&'r Source>) {
        match self {
            Error::UnexpectedToken(token) => sources.extend(token.src.as_ref()),

            Error::Tokenizer(_) => (),
            Error::EndOfStream => (),
        }
    }
}

impl From<tokenizer::Error> for Error {
    fn from(from: tokenizer::Error) -> Self {
        match from {
            tokenizer::Error::EndOfStream => Error::EndOfStream,
            error => Error::Tokenizer(error),
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
