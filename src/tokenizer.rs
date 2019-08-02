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
                        '"' => {
                            self.state = TokenState::String;
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
                TokenState::String => {
                    match c {
                        '"' => {
                            self.state = TokenState::Nothing;

                            let token = Token::String(self.token.clone());
                            self.token.clear();

                            return Some(token);
                        }
                        c => {
                            self.token.push(c);
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
    String,
    Word,
}

pub enum Token {
    String(String),
    Word(String),
}
