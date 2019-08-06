use std::io;

use crate::{
    context,
    evaluator::Evaluator,
    parser::Parser,
    reader::Reader,
    tokenizer::Tokenizer,
};


pub fn run<Program>(program: Program) -> Result<(), context::Error>
    where Program: io::Read
{
    let reader    = Reader::new(program);
    let tokenizer = Tokenizer::new(reader);
    let parser    = Parser::new(tokenizer);

    let mut evaluator = Evaluator::new();
    evaluator.run(parser)
}
