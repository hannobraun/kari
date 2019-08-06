use std::io;

use crate::{
    evaluator::Evaluator,
    parser::Parser,
    reader::Reader,
    tokenizer::Tokenizer,
};


pub fn run<Program>(program: Program)
    where Program: io::Read
{
    let reader    = Reader::new(program);
    let tokenizer = Tokenizer::new(reader);
    let parser    = Parser::new(tokenizer);
    let evaluator = Evaluator::new();

    if let Err(error) = evaluator.run(parser) {
        print!("\nERROR: {}\n", error);
        if let Some(span) = error.span() {
            print!("{:?}\n", span);
        }
        print!("\n");
    }
}
