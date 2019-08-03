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

use crate::iter::ErrorIter;


pub struct Reader;

impl Reader {
    pub fn read<R>(reader: R) -> ErrorIter<Chars<R>> where R: Read {
        ErrorIter::new(
            Chars {
                reader,
                buffer: [0; 4],
                index:  0,
            }
        )
    }
}


pub struct Chars<R> {
    reader: R,
    buffer: [u8; 4],
    index:  usize,
}

impl<R> Iterator for Chars<R> where R: Read {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = loop {
            if self.index >= self.buffer.len() {
                // This can only happen if an error occured before.
                return None;
            }

            let result = self.reader.read_exact(
                &mut self.buffer[self.index ..= self.index]
            );
            self.index += 1;

            match result {
                Ok(()) => (),
                Err(error) => {
                    match error.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            self.index = 0;
                            return None;
                        }
                        _ => {
                            return Some(Err(Error::Io(error)));
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
                            return Some(Err(Error::Utf8(error)));
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
        Some(Ok(c))
    }
}


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8(Utf8Error),
}
