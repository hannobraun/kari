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
        let mut state = State::Initial;

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
                        c => {
                            if !c.is_whitespace() {
                                state = State::Word;
                                token.push(c);
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
                            return Ok(Token {
                                kind: TokenKind::String(token),
                            });
                        }
                        c => {
                            token.push(c);
                        }
                    }
                }
                State::StringEscape => {
                    match c.c {
                        'n' => {
                            token.push('\n');
                            state = State::String;
                        }
                        c => {
                            return Err(Error::UnexpectedEscape(c));
                        }
                    }
                }
                State::Word => {
                    if c.is_whitespace() {
                        match token.as_str() {
                            "[" => return Ok(Token { kind: TokenKind::ListOpen }),
                            "]" => return Ok(Token { kind: TokenKind::ListClose }),

                            _ => {
                                if let Ok(number) = token.parse::<u32>() {
                                    return Ok(Token {
                                        kind: TokenKind::Number(number),
                                    });
                                }

                                return Ok(Token {
                                    kind: TokenKind::Word(token),
                                });
                            }
                        }
                    }

                    token.push(c.c);
                }
            }
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
