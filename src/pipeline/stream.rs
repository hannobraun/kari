pub trait Stream {
    type Item;
    type Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error>;
}

impl<T> Stream for &'_ mut T where T: Stream {
    type Item  = T::Item;
    type Error = T::Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        <T as Stream>::next(self)
    }
}
