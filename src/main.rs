fn main() {
    let mut program = include_str!("../examples/hello_world.kr").chars();

    let mut state = State::Nothing;

    let mut tokens  = Vec::new();
    let mut current = String::new();

    while let Some(c) = program.next() {
        match state {
            State::Nothing => {
                match c {
                    '"' => {
                        state = State::String;
                    }
                    c if c.is_whitespace() => {
                        ()
                    }
                    c => {
                        state = State::Word;
                        current.push(c);
                    }
                }
            }
            State::String => {
                match c {
                    '"' => {
                        state = State::Nothing;
                        tokens.push(Token::String(current.clone()));
                        current.clear();
                    }
                    c => {
                        current.push(c);
                    }
                }
            }
            State::Word => {
                match c {
                    c if c.is_whitespace() => {
                        state = State::Nothing;
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


enum State {
    Nothing,
    String,
    Word,
}

enum Token {
    String(String),
    Word(String),
}
