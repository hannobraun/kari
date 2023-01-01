pub mod parser;
pub mod reader;
pub mod tokenizer;

pub use self::{parser::Parser, reader::Reader, tokenizer::Tokenizer};

use std::io;

pub fn new<Stream>(name: String, stream: Stream) -> Parser<Tokenizer<Stream>>
where
    Stream: io::Read,
{
    let reader = Reader::new(stream);
    let tokenizer = Tokenizer::new(reader, name);
    Parser::new(tokenizer)
}

pub trait Stage {
    type Item;
    type Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error>;
}

impl<T> Stage for &'_ mut T
where
    T: Stage,
{
    type Item = T::Item;
    type Error = T::Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        <T as Stage>::next(self)
    }
}
