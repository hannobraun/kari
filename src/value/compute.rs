use crate::pipeline::tokenizer::{source::Merge, Source};

use super::Value;

pub trait Compute {
    type In;

    fn compute<Out, F, R>(self, f: F) -> Out
    where
        Out: Value<Inner = R>,
        F: FnOnce(Self::In) -> R;
}

impl<T> Compute for T
where
    T: Value,
{
    type In = T::Inner;

    fn compute<Out, F, R>(self, f: F) -> Out
    where
        Out: Value<Inner = R>,
        F: FnOnce(Self::In) -> R,
    {
        let (inner, span) = self.open();
        Out::new(f(inner), span)
    }
}

impl<A, B> Compute for (A, B)
where
    A: Value,
    B: Value,
{
    type In = (A::Inner, B::Inner);

    fn compute<Out, F, R>(self, f: F) -> Out
    where
        Out: Value<Inner = R>,
        F: FnOnce(Self::In) -> R,
    {
        let (a_inner, a_span) = self.0.open();
        let (b_inner, b_span) = self.1.open();
        Out::new(
            f((a_inner, b_inner)),
            Some(a_span).merge(Some(b_span)).unwrap_or(Source::Null),
        )
    }
}
