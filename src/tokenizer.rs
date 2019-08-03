use std::fmt;

use crate::iter::ErrorIter;


pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize<Chars>(chars: Chars) -> ErrorIter<Tokens<Chars>> {
        ErrorIter::new(
            Tokens {
                chars,
            }
        )
    }
}


pub struct Tokens<Chars> {
    chars: Chars
}

impl<Chars> Iterator for Tokens<Chars> where Chars: Iterator<Item=char> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut token = String::new();

        let start = self.chars.find(|c| !c.is_whitespace())?;

        if start == '"' {
            consume_string(&mut token, self.chars.by_ref());
            return Some(Ok(Token::String(token)));
        }

        token.push(start);
        token.extend(
            self.chars
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
    }
}


fn consume_string<S>(token: &mut String, mut string: S)
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
                    panic!("Unexpected escape sequence: {}", c);
                }
            }
        }
        else {
            match c {
                '"'  => return,
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
pub enum Error {}
