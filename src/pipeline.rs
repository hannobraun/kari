pub mod parser;
pub mod reader;
pub mod tokenizer;

pub use self::{parser::Parser, reader::Reader, tokenizer::Tokenizer};

use std::io;

pub fn new<Stream>(name: String, stream: Stream) -> Parser<Stream>
where
    Stream: io::Read,
{
    let reader = Reader::new(stream);
    let tokenizer = Tokenizer::new(reader, name);
    Parser::new(tokenizer)
}
