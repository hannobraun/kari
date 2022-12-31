pub mod expression;

pub use self::expression::Expression;

use std::fmt;

use crate::pipeline::{
    self,
    tokenizer::{self, token, Source, Token},
};

pub struct Parser<Tokenizer> {
    tokenizer: Tokenizer,
}

impl<Tokenizer> Parser<Tokenizer> {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Parser { tokenizer }
    }
}

impl<Tokenizer> pipeline::Stage for Parser<Tokenizer>
where
    Tokenizer: pipeline::Stage<Item = Token, Error = tokenizer::Error>,
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

impl<Tokenizer> Parser<Tokenizer>
where
    Tokenizer: pipeline::Stage<Item = Token, Error = tokenizer::Error>,
{
    fn parse_list(
        &mut self,
        mut list_source: Source,
    ) -> Result<Expression, Error> {
        let mut expressions = Vec::new();

        loop {
            let token = self.tokenizer.next()?;

            list_source = list_source.merge(Some(token.src.clone()));

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
            Error::UnexpectedToken(token) => sources.push(&token.src),

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
