use crate::{
    data::{
        ch::Char,
        span::Span,
        token::{
            self,
            Token,
        },
    },
    pipeline::{
        self,
        reader,
    },
};


pub struct Tokenizer<Reader> {
    reader: Reader,
    stream: String,
}

impl<Reader> Tokenizer<Reader> {
    pub fn new(reader: Reader, stream: String) -> Self {
        Self {
            reader,
            stream,
        }
    }
}

impl<Reader> pipeline::Stage for Tokenizer<Reader>
    where Reader: pipeline::Stage<Item=Char, Error=reader::Error>
{
    type Item  = Token;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        let mut builder = TokenBuilder::new(self.stream.clone());
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


struct TokenBuilder {
    buffer: String,
    stream: Option<String>,
    span:   Option<Span>,
}

impl TokenBuilder {
    fn new(stream: String) -> Self {
        Self {
            buffer: String::new(),
            stream: Some(stream),
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
                        stream: self.stream.take().unwrap(),
                        start:  c.pos,
                        end:    c.pos,
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
            kind: token::Kind::String(self.buffer),
            span: self.span.unwrap(),
        }
    }

    fn into_symbol(self) -> Token {
        Token {
            kind: token::Kind::Symbol(self.buffer),
            span: self.span.unwrap(),
        }
    }

    fn into_word(self) -> Token {
        let kind = match self.buffer.as_str() {
            "[" => token::Kind::ListOpen,
            "]" => token::Kind::ListClose,

            _ => {
                if let Ok(number) = self.buffer.parse::<u32>() {
                    token::Kind::Number(number)
                }
                else {
                    token::Kind::Word(self.buffer)
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
    Symbol,
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
