use std::io;


pub trait Stream : io::Read + io::Seek {}

impl<T> Stream for T where T: io::Read + io::Seek {}
