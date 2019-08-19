use crate::data::expr::{
    Any,
    TypeError
};


pub trait Type {
    const NAME: &'static str;

    type Value;

    fn from_any(_: Any) -> Result<Self::Value, Any>;

    fn check(any: Any) -> Result<Self::Value, TypeError>

    {
        Self::from_any(any)
            .map_err(|expression|
                TypeError {
                    expected: Self::NAME,
                    actual:   expression,
                }
            )
    }
}
