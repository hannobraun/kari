use std::fmt;

use crate::tokenizer::Token;


pub struct Interpreter {
    states: Vec<State>,
    stack:  Vec<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            states: vec![State::TopLevel],
            stack:  Vec::new(),
        }
    }

    pub fn run<Tokens>(&mut self, tokens: Tokens) -> Result<(), Error>
        where Tokens: IntoIterator<Item=Token>
    {
        for token in tokens {
            // Can't panic, as we have at least the top-level state on the state
            // stack.
            let state = self.states.last_mut().unwrap();

            match state {
                State::TopLevel => {
                    match token {
                        Token::QuoteOpen => {
                            self.states.push(State::Quote(Vec::new()));
                        }
                        Token::QuoteClose => {
                            return Err(Error::UnexpectedToken(token));
                        }
                        Token::String(string) => {
                            self.stack.push(Value::String(string));
                        }
                        Token::Word(word) => {
                            match word.as_str() {
                                "print" => {
                                    let arg = self.stack.pop().unwrap();
                                    print!("{}", arg);
                                }
                                "run" => {
                                    let arg = self.stack.pop().unwrap();
                                    match arg {
                                        Value::Quote(quote) => {
                                            self.run(quote)?;
                                        }
                                        arg => {
                                            return Err(Error::TypeError {
                                                expected: "quote",
                                                actual:   arg,
                                            });
                                        }
                                    };
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
                State::Quote(quote) => {
                    match token {
                        Token::QuoteOpen => {
                            self.states.push(State::Quote(Vec::new()));
                        }
                        Token::QuoteClose => {
                            self.stack.push(Value::Quote(quote.clone()));
                            self.states.pop();
                        }
                        token => {
                            quote.push(token);
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
    Quote(Vec<Token>),
}


pub enum Value {
    Quote(Vec<Token>),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Quote(quote) => {
                write!(f, "[ ")?;
                for value in quote {
                    write!(f, "{} ", value)?;
                }
                write!(f, "]")?;
            }
            Value::String(string) => {
                write!(f, "{}", string)?;
            }
        }

        Ok(())
    }
}


pub enum Error {
    UnexpectedToken(Token),
    UnexpectedWord(String),
    TypeError {
        expected: &'static str,
        actual:   Value,
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;

        match self {
            Error::UnexpectedToken(token) => {
                write!(f, "Unexpected token: \"{}\"", token)?;
            }
            Error::UnexpectedWord(word) => {
                write!(f, "Unexpected word: \"{}\"", word)?;
            }
            Error::TypeError { expected, actual } => {
                write!(
                    f,
                    "Expected value of type \"{}\", found {}",
                    expected,
                    actual,
                )?;
            }
        }

        write!(f, "\n\n")?;

        Ok(())
    }
}
