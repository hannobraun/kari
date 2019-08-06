use std::{
    fmt,
    io,
};

use crate::{
    reader::{
        self,
        Char,
        Position,
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
                            builder.process(c);
                        }
                        _ => {
                            if !c.is_whitespace() {
                                state = State::Word;
                                builder.store(c);
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
                            builder.process(c);
                        }
                        '"' => {
                            return Ok(builder.into_string());
                        }
                        _ => {
                            builder.store(c);
                        }
                    }
                }
                State::StringEscape => {
                    match c.c {
                        'n' => {
                            builder.store(Char { c: '\n', .. c });
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

                    builder.store(c);
                }
            }
        }
    }
}


struct TokenBuilder {
    buffer: String,
    span:   Option<Span>,
}

impl TokenBuilder {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            span:   None,
        }
    }

    fn process(&mut self, c: Char) {
        match &mut self.span {
            Some(span) => {
                span.end = c.pos
            }
            None => {
                self.span = Some(
                    Span {
                        start: c.pos,
                        end:   c.pos,
                    }
                )
            }
        }
    }

    fn store(&mut self, c: Char) {
        self.process(c);
        self.buffer.push(c.c);
    }

    fn into_string(self) -> Token {
        Token {
            kind: TokenKind::String(self.buffer),
            span: self.span.unwrap(),
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
            span: self.span.unwrap(),
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
    pub span: Span,
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


#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: Position,
    pub end:   Position,
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
