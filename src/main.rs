mod interpreter;
mod reader;
mod tokenizer;


use std::{
    env,
    fs::File,
};


fn main() {
    let program = env::args().skip(1).next().unwrap();
    let file    = format!("examples/{}.kr", program);
    let program = File::open(file).unwrap();

    let mut reader      = reader::Reader::new(program);
    let     tokenizer   = tokenizer::Tokenizer::new(&mut reader);
    let     interpreter = interpreter::Interpreter::new(tokenizer);

    let result = interpreter.run();

    if let Some(error) = reader.error() {
        print!("{:?}", error);
    }
    if let Err(error) = result {
        print!("{}", error);
    }
}
