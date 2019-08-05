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
            Token::ListOpen => {
                Expression::List(self.parse_list()?)
            }
            token @ Token::ListClose => {
                return Err(Error::UnexpectedToken(token));
            }

            Token::Number(number) => Expression::Number(number),
            Token::String(string) => Expression::String(string),
            Token::Word(word)     => Expression::Word(word),
        };

        Ok(expression)
    }

    pub fn parse_list(&mut self) -> Result<List, Error> {
        let mut list = List::new();

        loop {
            let expression = match self.tokenizer.next()? {
                Token::ListOpen => {
                    Expression::List(self.parse_list()?)
                }
                Token::ListClose => {
                    return Ok(list);
                }

                Token::Number(number) => Expression::Number(number),
                Token::String(string) => Expression::String(string),
                Token::Word(word)     => Expression::Word(word),
            };

            list.0.push(expression);
        }
    }
}


#[derive(Clone, Debug)]
pub enum Expression {
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Number(0)
    }
}


pub type Number = u32;


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
