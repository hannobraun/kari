use std::{
    fmt,
    io,
};

use crate::{
    reader::{
        self,
        Char,
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
        let mut builder = TokenBuilder::new();
        let mut state   = State::Initial;

        loop {
            let c = self.reader.next()?;

            match state {
                State::Initial => {
                    match c.c {
                        '#' => {
                            state = State::Comment;
                        }
                        '"' => {
                            state = State::String;
                        }
                        _ => {
                            if !c.is_whitespace() {
                                state = State::Word;
                                builder.push(c);
                            }
                        }
                    }
                }
                State::Comment => {
                    if c == '\n' {
                        state = State::Initial;
                    }
                }
                State::String => {
                    match c.c {
                        '\\' => {
                            state = State::StringEscape;
                        }
                        '"' => {
                            return Ok(builder.into_string());
                        }
                        _ => {
                            builder.push(c);
                        }
                    }
                }
                State::StringEscape => {
                    match c.c {
                        'n' => {
                            builder.push(Char { c: '\n', .. c });
                            state = State::String;
                        }
                        c => {
                            return Err(Error::UnexpectedEscape(c));
                        }
                    }
                }
                State::Word => {
                    if c.is_whitespace() {
                        return Ok(builder.into_word());
                    }

                    builder.push(c);
                }
            }
        }
    }
}


struct TokenBuilder {
    buffer: String,
}

impl TokenBuilder {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    fn push(&mut self, c: Char) {
        self.buffer.push(c.c);
    }

    fn into_string(self) -> Token {
        Token {
            kind: TokenKind::String(self.buffer),
        }
    }

    fn into_word(self) -> Token {
        let kind = match self.buffer.as_str() {
            "[" => TokenKind::ListOpen,
            "]" => TokenKind::ListClose,

            _ => {
                if let Ok(number) = self.buffer.parse::<u32>() {
                    TokenKind::Number(number)
                }
                else {
                    TokenKind::Word(self.buffer)
                }
            }
        };

        Token {
            kind,
        }
    }
}


enum State {
    Initial,
    Comment,
    String,
    StringEscape,
    Word,
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
