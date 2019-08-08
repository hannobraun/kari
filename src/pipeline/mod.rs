pub mod parser;
pub mod reader;
pub mod tokenizer;


use std::io;

use parser::Parser;
use reader::Reader;
use tokenizer::Tokenizer;


pub fn new<Stream>(stream: Stream) -> Parser<Tokenizer<Reader<Stream>>>
    where Stream: io::Read
{
    let reader    = Reader::new(stream);
    let tokenizer = Tokenizer::new(reader);
    let parser    = Parser::new(tokenizer);

    parser
}


pub trait Stage {
    type Item;
    type Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error>;
}

impl<T> Stage for &'_ mut T where T: Stage {
    type Item  = T::Item;
    type Error = T::Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        <T as Stage>::next(self)
    }
}
