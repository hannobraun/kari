use std::{
    fmt,
    io,
};

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
        let token = self.tokenizer.next()?;

        let expression = match token {
            Token::ListOpen => {
                Expression::List(self.parse_list()?)
            }
            token @ Token::ListClose => {
                return Err(Error::UnexpectedToken(token));
            }

            Token::Number(number) => Expression::Number(Number(number)),
            Token::String(string) => Expression::String(string),
            Token::Word(word)     => Expression::Word(word),
        };

        Ok(expression)
    }

    pub fn parse_list(&mut self) -> Result<List, Error> {
        let mut list = List::new();

        loop {
            let token = self.tokenizer.next()?;

            let expression = match token {
                Token::ListOpen => {
                    Expression::List(self.parse_list()?)
                }
                Token::ListClose => {
                    return Ok(list);
                }

                Token::Number(number) => Expression::Number(Number(number)),
                Token::String(string) => Expression::String(string),
                Token::Word(word)     => Expression::Word(word),
            };

            list.0.push(expression);
        }
    }
}


#[derive(Clone, Debug)]
pub enum Expression {
    Bool(Bool),
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Number(Number(0))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Bool(b)        => b.0.fmt(f),
            Expression::Number(number) => number.0.fmt(f),
            Expression::List(list)     => list.fmt(f),
            Expression::String(string) => string.fmt(f),
            Expression::Word(word)     => word.fmt(f),
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
            write!(f, "{} ", item)?;
        }
        write!(f, "]")?;

        Ok(())
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
