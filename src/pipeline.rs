pub mod parser;
pub mod reader;
pub mod tokenizer;

pub use self::{parser::Parser, reader::Reader, tokenizer::Tokenizer};

use std::io;

pub struct Pipeline<R> {
    pub parser: Parser<R>,
}

pub fn new<R>(name: String, stream: R) -> Pipeline<R>
where
    R: io::Read,
{
    let reader = Reader::new(stream);
    let tokenizer = Tokenizer::new(reader, name);
    let parser = Parser::new(tokenizer);

    Pipeline { parser }
}
