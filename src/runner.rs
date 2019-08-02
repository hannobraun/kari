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
    let mut reader      = Reader::new(program);
    let     tokenizer   = Tokenizer::new(&mut reader);
    let     interpreter = Interpreter::new(tokenizer);

    let result = interpreter.run();

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
