use crate::{
    data::{
        span::Span,
        token::{
            Token,
            TokenKind,
        },
    },
    pipeline::{
        self,
        reader::{
            self,
            Char,
        },
    },
};


pub struct Tokenizer<Reader> {
    reader: Reader,
}

impl<Reader> Tokenizer<Reader> {
    pub fn new(reader: Reader) -> Self {
        Self {
            reader,
        }
    }
}

impl<Reader> pipeline::Stage for Tokenizer<Reader>
    where Reader: pipeline::Stage<Item=Char, Error=reader::Error>
{
    type Item  = Token;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
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
                            builder.process(c);
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
