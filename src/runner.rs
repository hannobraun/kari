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
    let chars  = Reader::read(program);
    let tokens = Tokenizer::tokenize(chars);

    let mut interpreter = Interpreter::new();
    interpreter.run(tokens)
}
