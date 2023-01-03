pub mod expression;

pub use self::expression::Expression;

use std::{fmt, io};

use crate::pipeline::tokenizer::{self, token, Span, Token};

use super::{reader, tokenizer::span::Merge, Tokenizer};

pub struct Parser<R> {
    tokenizer: Tokenizer<R>,
}

impl<R> Parser<R> {
    pub fn new(tokenizer: Tokenizer<R>) -> Self {
        Parser { tokenizer }
    }
}

impl<R> Parser<R>
where
    R: io::Read,
{
    pub fn next_expression(
        &mut self,
        source: &mut String,
    ) -> Result<Expression, Error> {
        let token = self.tokenizer.next_token(source)?;

        let expr = match token.kind {
            token::Kind::ListOpen => self.parse_list(token.span, source)?,
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
        mut list_source: Option<Span>,
        source: &mut String,
    ) -> Result<Expression, Error> {
        let mut expressions = Vec::new();

        loop {
            let token = self.tokenizer.next_token(source)?;

            list_source = list_source.merge(token.span.clone());

            let expr = match token.kind {
                token::Kind::ListOpen => self.parse_list(token.span, source)?,
                token::Kind::ListClose => {
                    return Ok(Expression {
                        kind: expression::Kind::List(expressions),
                        span: list_source,
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
    pub fn sources<'r>(&'r self, sources: &mut Vec<&'r Span>) {
        match self {
            Error::UnexpectedToken(token) => {
                sources.extend(token.span.as_ref())
            }

            Error::Tokenizer(_) => (),
            Error::EndOfStream => (),
        }
    }
}

impl From<tokenizer::Error> for Error {
    fn from(from: tokenizer::Error) -> Self {
        match from {
            tokenizer::Error::Reader(reader::Error::EndOfStream) => {
                Error::EndOfStream
            }
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
