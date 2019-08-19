use crate::data::expr::Any;


pub trait Type {
    const NAME: &'static str;

    type Value;

    fn from_any(_: Any) -> Result<Self::Value, Any>;
}
