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

    pub fn run(mut self) {
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
                            panic!("Unexpected word: {}", word);
                        }
                    }
                }
            }
        }
    }
}
