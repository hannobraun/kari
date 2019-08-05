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

        let start = loop {
            let c = self.reader.find(|c| !c.is_whitespace())?;

            match c {
                '#' => {
                    self.reader.find(|c| c == '\n')?;
                }
                _ => {
                    break c;
                }
            }
        };

        if start == '"' {
            return consume_string(&mut token, &mut self.reader)
                .map(|()| Token { kind: TokenKind::String(token) });
        }

        token.push(start);
        self.reader.push_until(&mut token, |c| !c.is_whitespace())?;

        match token.as_str() {
            "[" => return Ok(Token { kind: TokenKind::ListOpen }),
            "]" => return Ok(Token { kind: TokenKind::ListClose }),

            _ => {
                if let Ok(number) = token.parse::<u32>() {
                    return Ok(Token { kind: TokenKind::Number(number) });
                }

                return Ok(Token { kind: TokenKind::Word(token) });
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


#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
}


#[derive(Clone, Debug)]
pub enum TokenKind {
    Number(u32),
    ListOpen,
    ListClose,
    String(String),
    Word(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Number(number) => number.fmt(f),
            TokenKind::ListOpen       => write!(f, "["),
            TokenKind::ListClose      => write!(f, "]"),
            TokenKind::String(string) => string.fmt(f),
            TokenKind::Word(word)     => word.fmt(f),
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
