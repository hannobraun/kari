use crate::value::types::{
    Downcast,
    TypeError,
};


/// Used to cast `value::Any` or tuples of `value::Any` to concrete values
///
/// This trait does mostly the same thing as `Downcast`, with the exception that
/// it's implemented on `value::Any` and tuples thereof, not type structs and
/// tuples thereof. This flips the API, allowing us to do things like
/// `(any1, any2).cast((t::Bool, t::Number))`.
pub trait Cast<T: Downcast> {
    fn cast(self, _: T) -> Result<<T as Downcast>::Output, TypeError>;
}

impl<T> Cast<T> for T::Input where T: Downcast {
    fn cast(self, t: T) -> Result<<T as Downcast>::Output, TypeError> {
        t.downcast(self)
    }
}


#[cfg(test)]
mod test {
    use decorum::R32;

    use crate::value::{
        self,
        t,
        v,
    };

    use super::Cast;


    #[test]
    fn it_should_cast_a_single_value() {
        let value = v::Bool::from(false);
        let any   = value::Any::from(value.clone());

        assert_eq!(any.clone().cast(t::Bool), Ok(value));
        assert!(any.cast(t::Number).is_err());
    }

    #[test]
    fn it_should_cast_two_values() {
        let v1 = v::Bool::from(false);
        let v2 = v::Number::from(0);

        let any1 = value::Any::from(v1.clone());
        let any2 = value::Any::from(v2.clone());

        assert_eq!(
            (any1.clone(), any2.clone()).cast((t::Bool, t::Number)),
            Ok((v1, v2))
        );
        assert!((any1, any2).cast((t::Bool, t::Bool)).is_err());
    }

    #[test]
    fn it_should_cast_three_values() {
        let v1 = v::Bool::from(false);
        let v2 = v::Number::from(0);
        let v3 = v::Float::from(R32::from_inner(0.0));

        let any1 = value::Any::from(v1.clone());
        let any2 = value::Any::from(v2.clone());
        let any3 = value::Any::from(v3.clone());

        assert_eq!(
            (any1.clone(), any2.clone(), any3.clone())
                .cast((t::Bool, t::Number, t::Float)),
            Ok((v1, v2, v3))
        );
        assert!((any1, any2, any3).cast((t::Bool, t::Bool, t::Float)).is_err());
    }
}
