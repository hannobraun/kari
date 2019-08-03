use std::io;

use crate::tokenizer::{
    self,
    Token,
    Tokenizer,
};


pub struct Parser<R> {
    tokenizer: Tokenizer<R>,
}

impl<R> Parser<R>
    where R: io::Read
{
    pub fn new(tokenizer: Tokenizer<R>) -> Self {
        Parser {
            tokenizer,
        }
    }

    pub fn next(&mut self) -> Result<Expression, Error> {
        let expression = match self.tokenizer.next()? {
            Token::QuoteOpen => {
                Expression::Quote(self.parse_quote()?)
            }
            token @ Token::QuoteClose => {
                return Err(Error::UnexpectedToken(token));
            }

            Token::Number(number) => Expression::Number(number),
            Token::String(string) => Expression::String(string),
            Token::Word(word)     => Expression::Word(word),
        };

        Ok(expression)
    }

    pub fn parse_quote(&mut self) -> Result<Quote, Error> {
        let mut quote = Quote::new();

        loop {
            let expression = match self.tokenizer.next()? {
                Token::QuoteOpen => {
                    Expression::Quote(self.parse_quote()?)
                }
                Token::QuoteClose => {
                    return Ok(quote);
                }

                Token::Number(number) => Expression::Number(number),
                Token::String(string) => Expression::String(string),
                Token::Word(word)     => Expression::Word(word),
            };

            quote.push(expression);
        }
    }
}


#[derive(Clone, Debug)]
pub enum Expression {
    Number(Number),
    Quote(Quote),
    String(String),
    Word(String),
}


pub type Number = u32;
pub type Quote  = Vec<Expression>;


#[derive(Debug)]
pub enum Error {
    Tokenizer(tokenizer::Error),
    UnexpectedToken(Token),
    EndOfStream,
}

impl From<tokenizer::Error> for Error {
    fn from(from: tokenizer::Error) -> Self {
        match from {
            tokenizer::Error::EndOfStream => Error::EndOfStream,
            error                         => Error::Tokenizer(error),
        }
    }
}
