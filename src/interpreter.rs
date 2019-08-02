use std::fmt;

use crate::tokenizer::Token;


pub struct Interpreter<Tokens> {
    tokens: Tokens,
    states: Vec<State>,
    stack:  Vec<String>,
}

impl<Tokens> Interpreter<Tokens> where Tokens: Iterator<Item=Token> {
    pub fn new(tokens: Tokens) -> Self {
        Interpreter {
            tokens,
            states: vec![State::TopLevel],
            stack:  Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        for token in self.tokens {
            // Can't panic, as we have at least the top-level state on the state
            // stack.
            let state = self.states.last_mut().unwrap();

            match state {
                State::TopLevel => {
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
                                    return Err(Error::UnexpectedWord(
                                        word.to_string())
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}


enum State {
    TopLevel,
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
