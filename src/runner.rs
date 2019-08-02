use std::io;

use crate::{
    interpreter::Interpreter,
    reader::Reader,
    tokenizer::Tokenizer,
};


pub fn run<Program>(program: Program) where Program: io::Read {
    let mut reader      = Reader::new(program);
    let     tokenizer   = Tokenizer::new(&mut reader);
    let     interpreter = Interpreter::new(tokenizer);

    let result = interpreter.run();

    if let Some(error) = reader.error() {
        print!("{:?}", error);
    }
    if let Err(error) = result {
        print!("{}", error);
    }
}
