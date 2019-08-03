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
    reader: R,
    buffer: [u8; 4],
    index:  usize,
    error:  Option<Error>,
}

impl<R> Reader<R> {
    pub fn read(reader: R) -> Self {
        Reader {
            reader,
            buffer: [0; 4],
            index:  0,
            error:  None,
        }
    }

    pub fn error(self) -> Option<Error> {
        self.error
    }
}

impl<'r, R> Iterator for &'r mut Reader<R> where R: Read {
    type Item = char;

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
                            ()
                        }
                        _ => {
                            self.error = Some(Error::Io(error));
                        }
                    }

                    return None;
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
                            self.error = Some(Error::Utf8(error));
                            return None;
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
        Some(c)
    }
}


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8(Utf8Error),
}
