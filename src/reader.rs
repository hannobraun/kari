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
    input: R,

    buffer: [u8; 4],
    index:  usize,

    next_column: usize,
    next_line:   usize,
}

impl<R> Reader<R> where R: Read {
    pub fn new(input: R) -> Self {
        Reader {
            input,

            buffer: [0; 4],
            index:  0,

            next_column: 0,
            next_line:   0,
        }
    }

    pub fn next(&mut self) -> Result<Char, Error> {
        let c = loop {
            if self.index >= self.buffer.len() {
                // This can only happen if an error occured before.
                return Err(Error::EndOfStream);
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
                            return Err(Error::EndOfStream);
                        }
                        _ => {
                            return Err(Error::Io(error).into());
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
                            return Err(Error::Utf8(error).into());
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

        let c = Char {
            c,
            column: self.next_column,
            line:   self.next_line,
        };

        self.next_column += 1;
        if c.c == '\n' {
            self.next_column = 0;
            self.next_line += 1;
        }

        Ok(c)
    }
}


#[derive(Clone, Copy)]
pub struct Char {
    pub c:      char,
    pub column: usize,
    pub line:   usize,
}

impl Char {
    pub fn is_whitespace(&self) -> bool {
        self.c.is_whitespace()
    }
}

impl PartialEq<char> for Char {
    fn eq(&self, other: &char) -> bool {
        self.c.eq(other)
    }
}


#[derive(Debug)]
pub enum Error {
    EndOfStream,
    Io(io::Error),
    Utf8(Utf8Error),
}
