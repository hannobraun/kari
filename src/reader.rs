use std::{
    io::{
        self,
        prelude::*,
    },
    str::{
        self,
        Utf8Error,
    },
};


pub struct Reader<R> {
    input:  R,
    buffer: [u8; 4],
    index:  usize,
}

impl<R> Reader<R> where R: Read {
    pub fn new(input: R) -> Self {
        Reader {
            input,
            buffer: [0; 4],
            index:  0,
        }
    }

    pub fn next(&mut self) -> Result<Option<char>, Error> {
        let c = loop {
            if self.index >= self.buffer.len() {
                // This can only happen if an error occured before.
                return Ok(None);
            }

            let result = self.input.read_exact(
                &mut self.buffer[self.index ..= self.index]
            );
            self.index += 1;

            match result {
                Ok(()) => (),
                Err(error) => {
                    match error.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            self.index = 0;
                            return Ok(None);
                        }
                        _ => {
                            return Err(Error::Io(error));
                        }
                    }
                }
            }

            match str::from_utf8(&self.buffer[.. self.index]) {
                Ok(s) => {
                    assert_eq!(s.chars().count(), 1);

                    // Can't panic. We just asserted that there is exactly one
                    // char.
                    break s.chars().next().unwrap();
                }
                Err(error) => {
                    match self.index {
                        i if i == 4 => {
                            return Err(Error::Utf8(error));
                        }
                        i if i < 4 => {
                            continue;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }
        };

        self.index = 0;
        Ok(Some(c))
    }

    pub fn find<P>(&mut self, predicate: P) -> Result<Option<char>, Error>
        where P: Fn(char) -> bool
    {
        while let Some(c) = self.next()? {
            if predicate(c) {
                return Ok(Some(c));
            }
        }

        Ok(None)
    }

    pub fn push_until<P>(&mut self, s: &mut String, predicate: P)
        -> Result<(), Error>
        where P: Fn(char) -> bool
    {
        while let Some(c) = self.next()? {
            if predicate(c) {
                s.push(c);
            }
            else {
                break;
            }
        }

        Ok(())
    }
}


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8(Utf8Error),
}
