use std::{
    fmt,
    io,
};

use crate::iter::ErrorIter;

use crate::{
    reader::{
        self,
        Reader,
    },
};


pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize<R>(reader: Reader<R>) -> ErrorIter<Tokens<R>>
        where R: io::Read
    {
        ErrorIter::new(
            Tokens {
                reader,
            }
        )
    }
}


pub struct Tokens<R> {
    reader: Reader<R>
}

impl<R> Iterator for Tokens<R>
    where R: io::Read
{
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut token = String::new();

        let start = match self.reader.find(|c| !c.is_whitespace()) {
            Ok(c)                           => c,
            Err(reader::Error::EndOfStream) => return None,
            Err(error)                      => return Some(Err(error.into())),
        };

        if start == '"' {
            let string = consume_string(&mut token, &mut self.reader)
                .map(|()| Token::String(token));

            return Some(string);
        }

        token.push(start);
        match self.reader.push_until(&mut token, |c| !c.is_whitespace()) {
            Ok(c)                           => c,
            Err(reader::Error::EndOfStream) => return None,
            Err(error)                      => return Some(Err(error.into())),
        }

        match token.as_str() {
            "[" => return Some(Ok(Token::QuoteOpen)),
            "]" => return Some(Ok(Token::QuoteClose)),

            _ => {
                if let Ok(number) = token.parse::<u32>() {
                    return Some(Ok(Token::Number(number)));
                }

                return Some(Ok(Token::Word(token)));
            }
        }
    }
}


fn consume_string<R>(token: &mut String, reader: &mut Reader<R>)
    -> Result<(), Error>
    where R: io::Read
{
    let mut escape = false;

    loop {
        let c = match reader.next() {
            Ok(c)                           => c,
            Err(reader::Error::EndOfStream) => return Ok(()),
            Err(error)                      => return Err(error.into()),
        };

        if escape {
            match c {
                'n' => {
                    token.push('\n');
                    escape = false;
                }
                c => {
                    return Err(Error::UnexpectedEscape(c));
                }
            }
        }
        else {
            match c {
                '"'  => return Ok(()),
                '\\' => escape = true,
                c    => token.push(c),
            }
        }
    }
}


#[derive(Clone, Debug)]
pub enum Token {
    Number(u32),
    QuoteOpen,
    QuoteClose,
    String(String),
    Word(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(number) => number.fmt(f),
            Token::QuoteOpen      => write!(f, "["),
            Token::QuoteClose     => write!(f, "]"),
            Token::String(string) => string.fmt(f),
            Token::Word(word)     => word.fmt(f),
        }
    }
}


#[derive(Debug)]
pub enum Error {
    Reader(reader::Error),
    UnexpectedEscape(char),
}

impl From<reader::Error> for Error {
    fn from(from: reader::Error) -> Self {
        Error::Reader(from)
    }
}
