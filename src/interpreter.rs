use std::fmt;

use crate::tokenizer::Token;


pub struct Interpreter<Tokens> {
    tokens: Tokens,
    stack:  Vec<String>,
}

impl<Tokens> Interpreter<Tokens> where Tokens: Iterator<Item=Token> {
    pub fn new(tokens: Tokens) -> Self {
        Interpreter {
            tokens,
            stack: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        for token in self.tokens {
            match token {
                Token::String(string) => {
                    self.stack.push(string);
                }
                Token::Word(word) => {
                    match word.as_str() {
                        "print" => {
                            let arg = self.stack.pop().unwrap();
                            print!("{}", arg);
                        }
                        word => {
                            return Err(Error::UnexpectedWord(word.to_string()));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}


pub enum Error {
    UnexpectedWord(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedWord(word) => {
                write!(f, "\nUnexpected word: \"{}\"\n\n", word)?;
            }
        }

        Ok(())
    }
}
