use std::io;

use crate::{
    evaluate,
    evaluator::Evaluator,
    parser::Parser,
    reader::Reader,
    tokenizer::Tokenizer,
};


pub fn run<Program>(program: Program) -> Result<(), evaluate::Error>
    where Program: io::Read
{
    let reader    = Reader::new(program);
    let tokenizer = Tokenizer::new(reader);
    let parser    = Parser::new(tokenizer);

    let mut interpreter = Evaluator::new();
    interpreter.run(parser)
}
