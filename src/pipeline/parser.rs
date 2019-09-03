use std::fmt;

use crate::{
    data::{
        expr,
        span::Span,
        token::{
            self,
            Token,
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
    type Item  = expr::Any;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        let token = self.tokenizer.next()?;

        let expr = match token.kind {
            token::Kind::ListOpen => {
                self.parse_list(token.span)?
            }
            token::Kind::ListClose => {
                return Err(Error::UnexpectedToken(token));
            }
            _ => {
                expr::Any::from_token(token)
            }
        };

        Ok(expr)
    }
}

impl<Tokenizer> Parser<Tokenizer>
    where Tokenizer: pipeline::Stage<Item=Token, Error=tokenizer::Error>
{
    fn parse_list(&mut self, mut list_span: Span) -> Result<expr::Any, Error> {
        let mut expressions = Vec::new();

        loop {
            let token = self.tokenizer.next()?;

            list_span = list_span.merge(&token.span);

            let expr = match token.kind {
                token::Kind::ListOpen => {
                    self.parse_list(token.span)?
                }
                token::Kind::ListClose => {
                    return Ok(
                        expr::Any {
                            kind: expr::Kind::List(expressions),
                            span: list_span,
                        }
                    );
                }
                _ => {
                    expr::Any::from_token(token)
                }
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
    pub fn spans(self, spans: &mut Vec<Span>) {
        match self {
            Error::UnexpectedToken(token) => spans.push(token.span),

            Error::Tokenizer(_) => (),
            Error::EndOfStream  => (),
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
