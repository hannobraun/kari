use crate::data::expr::{
    self,
    Any,
};


pub trait Type {
    const NAME: &'static str;

    type Value;

    fn from_any(_: Any) -> Result<Self::Value, Any>;

    fn check(any: Any) -> Result<Self::Value, expr::Error>

    {
        Self::from_any(any)
            .map_err(|expression|
                expr::Error::TypeError {
                    expected: Self::NAME,
                    actual:   expression,
                }
            )
    }
}
