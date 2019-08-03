use std::fmt;

use crate::iter::ErrorIter;

use crate::reader;


pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize<Chars>(chars: ErrorIter<Chars>) -> ErrorIter<Tokens<Chars>>
        where Chars: Iterator<Item=Result<char, reader::Error>>
    {
        ErrorIter::new(
            Tokens {
                chars,
            }
        )
    }
}


pub struct Tokens<Chars: Iterator> {
    chars: ErrorIter<Chars>,
}

impl<Chars> Iterator for Tokens<Chars>
    where Chars: Iterator<Item=Result<char, reader::Error>>
{
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.chars.take_until_error();

        chars.handle_error(|mut chars| {
            let mut token = String::new();

            let start = chars
                .find(|c| !c.is_whitespace())?;

            if start == '"' {
                let string = consume_string(&mut token, chars.by_ref())
                    .map(|()| Token::String(token));

                return Some(string);
            }

            token.push(start);
            token.extend(
                chars
                    .by_ref()
                    .take_while(|c| !c.is_whitespace())
            );

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
        })
    }
}


fn consume_string<S>(token: &mut String, mut string: S) -> Result<(), Error>
    where S: Iterator<Item=char>
{
    let mut escape = false;

    while let Some(c) = string.next() {
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

    Ok(())
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
