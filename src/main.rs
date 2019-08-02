mod interpreter;
mod tokenizer;


use std::{
    env,
    fs::File,
    io::prelude::*,
};


fn main() {
    let program = env::args().skip(1).next().unwrap();
    let file    = format!("examples/{}.kr", program);

    let mut program = String::new();
    File::open(file)
        .unwrap()
        .read_to_string(&mut program)
        .unwrap();

    let tokenizer   = tokenizer::Tokenizer::new(program.chars());
    let interpreter = interpreter::Interpreter::new(tokenizer);

    if let Err(error) = interpreter.run() {
        print!("{}", error);
    }
}
