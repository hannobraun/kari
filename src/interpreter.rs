use std::{
    borrow::Cow,
    io,
};

use crate::{
    error,
    evaluator::Evaluator,
    pipeline,
};


pub fn run<Stream>(name: Cow<str>, mut stream: Stream) -> bool
    where Stream: io::Read + io::Seek
{
    let pipeline = pipeline::new(
        name.clone().into_owned(),
        stream.by_ref(),
    );

    if let Err(error) = Evaluator::run(pipeline) {
        if let Err(error) = error::print(error, &name, stream) {
            print!("Error printing error: {}\n", error)
        }
        return false;
    }

    true
}
