mod interpreter;
mod tokenizer;


use std::{
    fs::File,
    io::prelude::*,
};


fn main() {
    let mut program = String::new();
    File::open("examples/hello_world.kr")
        .unwrap()
        .read_to_string(&mut program)
        .unwrap();

    let tokenizer   = tokenizer::Tokenizer::new(program.chars());
    let interpreter = interpreter::Interpreter::new(tokenizer);

    if let Err(error) = interpreter.run() {
        print!("{}", error);
    }
}
