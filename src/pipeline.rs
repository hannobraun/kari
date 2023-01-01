pub mod parser;
pub mod reader;
pub mod tokenizer;

pub use self::{parser::Parser, reader::Reader, tokenizer::Tokenizer};

use std::io;

pub fn new<R>(name: String, stream: R) -> Parser<R>
where
    R: io::Read,
{
    let reader = Reader::new(stream);
    let tokenizer = Tokenizer::new(reader, name);
    Parser::new(tokenizer)
}
