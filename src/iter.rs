pub struct ErrorIter<Iter> {
    iter:  Iter,
    error: bool,
}

impl<Iter> ErrorIter<Iter> {
    pub fn new(iter: Iter) -> Self {
        Self {
            iter,
            error: false,
        }
    }
}

impl<Iter, T, E> Iterator for ErrorIter<Iter>
    where Iter: Iterator<Item=Result<T, E>>
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error {
            return None;
        }

        let item = self.iter.next();

        if let Some(Err(error)) = item {
            self.error = true;
            return Some(Err(error));
        }

        item
    }
}
