use std::{
    fmt,
    io,
};

use crate::{
    reader::{
        self,
        Reader,
    },
};


pub struct Tokenizer<R> {
    reader: Reader<R>
}

impl<R> Tokenizer<R>
    where R: io::Read
{
    pub fn new(reader: Reader<R>) -> Self {
        Self {
            reader,
        }
    }

    pub fn next(&mut self) -> Result<Token, Error> {
        let mut token = String::new();

        let start = self.reader.find(|c| !c.is_whitespace())?;

        if start == '"' {
            return consume_string(&mut token, &mut self.reader)
                .map(|()| Token::String(token));
        }

        token.push(start);
        self.reader.push_until(&mut token, |c| !c.is_whitespace())?;

        match token.as_str() {
            "[" => return Ok(Token::ListOpen),
            "]" => return Ok(Token::ListClose),

            _ => {
                if let Ok(number) = token.parse::<u32>() {
                    return Ok(Token::Number(number));
                }

                return Ok(Token::Word(token));
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
    ListOpen,
    ListClose,
    String(String),
    Word(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(number) => number.fmt(f),
            Token::ListOpen       => write!(f, "["),
            Token::ListClose      => write!(f, "]"),
            Token::String(string) => string.fmt(f),
            Token::Word(word)     => word.fmt(f),
        }
    }
}


#[derive(Debug)]
pub enum Error {
    Reader(reader::Error),
    UnexpectedEscape(char),
    EndOfStream,
}

impl From<reader::Error> for Error {
    fn from(from: reader::Error) -> Self {
        match from {
            reader::Error::EndOfStream => Error::EndOfStream,
            error                      => Error::Reader(error),
        }
    }
}
