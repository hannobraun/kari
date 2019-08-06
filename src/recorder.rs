use crate::{
    reader::{
        self,
        Char,
    },
    stream::Stream,
};


pub struct Recorder<Reader> {
    reader: Reader,
    chars:  Vec<Char>,
}

impl<Reader> Recorder<Reader> {
    pub fn new(reader: Reader) -> Self {
        Self {
            reader,
            chars: Vec::new(),
        }
    }

    pub fn chars(self) -> Vec<Char> {
        self.chars
    }
}

impl<Reader> Stream for Recorder<Reader>
    where Reader: Stream<Item=Char, Error=reader::Error>
{
    type Item  = Reader::Item;
    type Error = Reader::Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        let c = self.reader.next()?;
        self.chars.push(c);
        Ok(c)
    }
}
