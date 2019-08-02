use std::fmt;


pub struct Tokenizer<Chars> {
    chars: Chars,
    state: TokenState,
    token: String,
}

impl<Chars> Tokenizer<Chars> {
    pub fn new(chars: Chars) -> Self {
        Tokenizer {
            chars,
            state: TokenState::Nothing,
            token: String::new(),
        }
    }
}

impl<Chars> Iterator for Tokenizer<Chars> where Chars: Iterator<Item=char> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.chars.next() {
            match self.state {
                TokenState::Nothing => {
                    match c {
                        '[' => {
                            return Some(Token::QuoteOpen);
                        }
                        ']' => {
                            return Some(Token::QuoteClose);
                        }
                        '"' => {
                            self.state = TokenState::String(StringState::Char);
                        }
                        c if c.is_whitespace() => {
                            ()
                        }
                        c => {
                            self.state = TokenState::Word;
                            self.token.push(c);
                        }
                    }
                }
                TokenState::String(StringState::Char) => {
                    match c {
                        '"' => {
                            self.state = TokenState::Nothing;

                            let token = Token::String(self.token.clone());
                            self.token.clear();

                            return Some(token);
                        }
                        '\\' => {
                            self.state = TokenState::String(
                                StringState::Escape
                            );
                        }
                        c => {
                            self.token.push(c);
                        }
                    }
                }
                TokenState::String(StringState::Escape) => {
                    match c {
                        'n' => {
                            self.token.push('\n');
                            self.state = TokenState::String(StringState::Char);
                        }
                        c => {
                            panic!("Unexpected escape sequence: {}", c);
                        }
                    }
                }
                TokenState::Word => {
                    match c {
                        c if c.is_whitespace() => {
                            self.state = TokenState::Nothing;

                            let token = Token::Word(self.token.clone());
                            self.token.clear();

                            return Some(token);
                        }
                        c => {
                            self.token.push(c);
                        }
                    }
                }
            }
        }

        None
    }
}


enum TokenState {
    Nothing,
    String(StringState),
    Word,
}

enum StringState {
    Char,
    Escape,
}


#[derive(Clone)]
pub enum Token {
    QuoteOpen,
    QuoteClose,
    String(String),
    Word(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::QuoteOpen      => write!(f, "["),
            Token::QuoteClose     => write!(f, "]"),
            Token::String(string) => write!(f, "{}", string),
            Token::Word(word)     => write!(f, "{}", word),
        }
    }
}
