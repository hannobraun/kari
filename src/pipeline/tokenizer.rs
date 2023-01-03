pub mod token;

use std::io;

pub use self::token::Token;

use crate::{pipeline::reader, source::{Char, Span}};

use super::Reader;

pub struct Tokenizer<R> {
    reader: Reader<R>,
    stream: String,
}

impl<R> Tokenizer<R> {
    pub fn new(reader: Reader<R>, stream: String) -> Self {
        Self { reader, stream }
    }
}

impl<R> Tokenizer<R>
where
    R: io::Read,
{
    pub fn next_token(&mut self, source: &mut String) -> Result<Token, Error> {
        let mut state = State::Initial;
        let mut builder = TokenBuilder::new(self.stream.clone());

        loop {
            let c = self.reader.next_char(source)?;

            match state {
                State::Initial => match c.c {
                    '#' => {
                        state = State::Comment;
                    }
                    '"' => {
                        state = State::String;
                        builder.process(c);
                    }
                    ':' => {
                        state = State::Symbol;
                        builder.process(c);
                    }
                    _ => {
                        if !c.is_whitespace() {
                            state = State::Word;
                            builder.store(c);
                        }
                    }
                },
                State::Comment => {
                    if c == '\n' {
                        state = State::Initial;
                    }
                }
                State::String => match c.c {
                    '\\' => {
                        state = State::StringEscape;
                        builder.process(c);
                    }
                    '"' => {
                        builder.process(c);
                        return Ok(builder.into_string());
                    }
                    _ => {
                        builder.store(c);
                    }
                },
                State::StringEscape => match c.c {
                    'n' => {
                        builder.store(Char { c: '\n', ..c });
                        state = State::String;
                    }
                    c => {
                        return Err(Error::UnexpectedEscapeCode(c));
                    }
                },
                State::Symbol => {
                    if c.is_whitespace() {
                        return Ok(builder.into_symbol());
                    }

                    builder.store(c);
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

enum State {
    Initial,
    Comment,
    String,
    StringEscape,
    Symbol,
    Word,
}

struct TokenBuilder {
    buffer: String,
    stream_name: Option<String>,
    span: Option<Span>,
}

impl TokenBuilder {
    fn new(stream_name: String) -> Self {
        Self {
            buffer: String::new(),
            stream_name: Some(stream_name),
            span: None,
        }
    }

    fn process(&mut self, c: Char) {
        match &mut self.span {
            Some(span) => span.end = c.pos,
            None => {
                self.span = Some(Span {
                    stream_name: self.stream_name.take().unwrap(),
                    start: c.pos,
                    end: c.pos,
                })
            }
        }
    }

    fn store(&mut self, c: Char) {
        self.process(c);
        self.buffer.push(c.c);
    }

    fn into_string(self) -> Token {
        Token {
            kind: token::Kind::String(self.buffer),
            span: self.span,
        }
    }

    fn into_symbol(self) -> Token {
        Token {
            kind: token::Kind::Symbol(self.buffer),
            span: self.span,
        }
    }

    fn into_word(self) -> Token {
        let kind = match self.buffer.as_str() {
            "[" => token::Kind::ListOpen,
            "]" => token::Kind::ListClose,

            _ => token::Kind::parse_word(self.buffer),
        };

        Token {
            kind,
            span: self.span,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Reader error")]
    Reader(#[from] reader::Error),

    #[error("Unexpected escape code: {0}")]
    UnexpectedEscapeCode(char),
}
