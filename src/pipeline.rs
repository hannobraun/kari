pub mod parser;
pub mod reader;
pub mod tokenizer;

pub use self::{parser::Parser, reader::Reader, tokenizer::Tokenizer};

pub struct Pipeline<R> {
    pub parser: Parser<R>,
}

impl<R> Pipeline<R> {
    pub fn new(name: String, stream: R) -> Self {
        let reader = Reader::new(stream);
        let tokenizer = Tokenizer::new(reader, name);
        let parser = Parser::new(tokenizer);

        Pipeline { parser }
    }
}
