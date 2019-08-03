pub type Result<T, E> = std::result::Result<T, Error<E>>;


pub enum Error<T> {
    EndOfStream,
    Other(T),
}

impl<T> From<T> for Error<T> {
    fn from(from: T) -> Self {
        Error::Other(from)
    }
}
