use crate::{
    functions::{
        Function,
        Functions,
    },
    stack::{
        self,
        Quote,
        Stack,
        Value,
    },
    tokenizer::Token,
};


pub struct Interpreter {
    states:    Vec<State>,
    stack:     Stack,
    functions: Functions,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            states:    vec![State::TopLevel],
            stack:     Stack::new(),
            functions: Functions::new(),
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
                        Token::Number(number) => {
                            self.stack.push(Value::Number(number));
                        }
                        Token::String(string) => {
                            self.stack.push(Value::String(string));
                        }
                        Token::Word(word) => {
                            match word.as_str() {
                                "run" => {
                                    let arg = self.stack.pop()?;
                                    match arg {
                                        Value::Quote(quote) => {
                                            self.run(quote)?;
                                        }
                                        arg => {
                                            return Err(
                                                Error::Stack(
                                                    stack::Error::TypeError {
                                                        expected: "quote",
                                                        actual:   arg,
                                                    }
                                                )
                                            );
                                        }
                                    };
                                }
                                word => {
                                    match self.functions.get(word) {
                                        Some(Function::Builtin(builtin)) => {
                                            builtin(
                                                &mut self.stack,
                                                &mut self.functions,
                                            )?;
                                        }
                                        Some(Function::Quote(quote)) => {
                                            self.run(quote)?;
                                        }
                                        None => {
                                            return Err(Error::UnknownFunction(
                                                word.to_string())
                                            );
                                        }
                                    }
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
    Quote(Quote),
}


#[derive(Debug)]
pub enum Error {
    UnexpectedToken(Token),
    UnknownFunction(String),
    Stack(stack::Error),
}

impl From<stack::Error> for Error {
    fn from(from: stack::Error) -> Self {
        Error::Stack(from)
    }
}