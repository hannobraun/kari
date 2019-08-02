fn main() {
    let mut program = include_str!("../examples/hello_world.kr").chars();

    let mut state = TokenState::Nothing;

    let mut tokens  = Vec::new();
    let mut current = String::new();

    while let Some(c) = program.next() {
        match state {
            TokenState::Nothing => {
                match c {
                    '"' => {
                        state = TokenState::String;
                    }
                    c if c.is_whitespace() => {
                        ()
                    }
                    c => {
                        state = TokenState::Word;
                        current.push(c);
                    }
                }
            }
            TokenState::String => {
                match c {
                    '"' => {
                        state = TokenState::Nothing;
                        tokens.push(Token::String(current.clone()));
                        current.clear();
                    }
                    c => {
                        current.push(c);
                    }
                }
            }
            TokenState::Word => {
                match c {
                    c if c.is_whitespace() => {
                        state = TokenState::Nothing;
                        tokens.push(Token::Word(current.clone()));
                        current.clear();
                    }
                    c => {
                        current.push(c);
                    }
                }
            }
        }
    }

    let mut stack = Vec::new();

    for token in tokens {
        match token {
            Token::String(string) => {
                stack.push(string);
            }
            Token::Word(word) => {
                match word.as_str() {
                    "print" => {
                        let arg = stack.pop().unwrap();
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


enum TokenState {
    Nothing,
    String,
    Word,
}

enum Token {
    String(String),
    Word(String),
}
