use std::io;

use crate::{
    interpreter::{
        self,
        Interpreter,
    },
    reader::{
        self,
        Reader,
    },
    tokenizer::Tokenizer,
};


pub fn run<Program>(program: Program) -> Result<(), Error>
    where Program: io::Read
{
    let mut reader      = Reader::read(program);
    let     tokenizer   = Tokenizer::tokenize(&mut reader);
    let mut interpreter = Interpreter::new();

    let result = interpreter.run(tokenizer);

    if let Some(error) = reader.error() {
        return Err(Error::Reader(error));
    }
    if let Err(error) = result {
        return Err(Error::Interpreter(error));
    }

    Ok(())
}


pub enum Error {
    Reader(reader::Error),
    Interpreter(interpreter::Error),
}
