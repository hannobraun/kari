use std::io;

use crate::{
    interpreter::{
        self,
        Interpreter,
    },
    reader::Reader,
    tokenizer::Tokenizer,
};


pub fn run<Program>(program: Program) -> Result<(), interpreter::Error>
    where Program: io::Read
{
    let reader    = Reader::new(program);
    let tokenizer = Tokenizer::new(reader);

    let mut interpreter = Interpreter::new();
    interpreter.run(tokenizer)
}
