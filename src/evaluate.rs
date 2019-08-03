use crate::{
    parser::{
        self,
        Expression,
    },
    stack,
};


pub trait Evaluate {
    fn evaluate(&mut self, expressions: &mut Iterator<Item=Expression>)
        -> Result<(), Error>;
}


#[derive(Debug)]
pub enum Error {
    Parser(parser::Error),
    UnknownFunction(String),
    Stack(stack::Error),
}
