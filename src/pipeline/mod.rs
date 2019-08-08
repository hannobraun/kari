pub mod evaluator;
pub mod parser;
pub mod reader;
pub mod tokenizer;


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
