mod tokenizer;


use tokenizer::{
    Token,
    Tokenizer,
};


fn main() {
    let program = include_str!("../examples/hello_world.kr");

    let mut stack = Vec::new();

    for token in Tokenizer::new(program.chars()) {
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
