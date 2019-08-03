pub struct ErrorIter<Iter: Iterator> {
    iter:   Iter,
    peeked: Option<Iter::Item>,
    error:  bool,
}

impl<Iter> ErrorIter<Iter> where Iter: Iterator {
    pub fn new(iter: Iter) -> Self {
        Self {
            iter,
            peeked: None,
            error:  false,
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

        let item = self.peeked.take().or_else(|| self.iter.next());

        if let Some(Err(error)) = item {
            self.error = true;
            return Some(Err(error));
        }

        item
    }
}

impl<Iter, T, E> ErrorIter<Iter> where Iter: Iterator<Item=Result<T, E>> {
    pub fn peek(&mut self) -> Option<&Iter::Item> {
        self.peeked = self.iter.next();
        self.peeked.as_ref()
    }

    pub fn take_until_error(&mut self) -> TakeUntilError<Iter, T, E> {
        TakeUntilError::new(self)
    }
}


pub struct TakeUntilError<'r, Iter: Iterator<Item=Result<T, E>>, T, E>(
    &'r mut ErrorIter<Iter>
);

impl<'r, Iter, T, E> TakeUntilError<'r, Iter, T, E>
    where Iter: Iterator<Item=Result<T, E>>
{
    pub fn new(iter: &'r mut ErrorIter<Iter>) -> Self {
        TakeUntilError(iter)
    }
}

impl<'r, Iter, T, E> Iterator for TakeUntilError<'r, Iter, T, E>
    where Iter: Iterator<Item=Result<T, E>>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Err(_)) = self.0.peek() {
            return None;
        }

        self.0
            .next()
            // Can't panic. We just peeked at the next item, and if it were an
            // error, we would have returned already.
            .map(|item| item.unwrap_or_else(|_| unreachable!()))
    }
}