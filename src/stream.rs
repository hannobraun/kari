pub trait Stream {
    type Item;
    type Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error>;
}
