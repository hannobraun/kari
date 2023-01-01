mod char;
mod position;

pub use self::{char::Char, position::Position};

use std::{
    io,
    str::{self, Utf8Error},
};

use crate::pipeline;

/// Converts a stream of bytes into a stream of UTF-8 characters
pub struct Reader<R> {
    input: R,

    buffer: [u8; 4],
    buffer_i: usize,

    next_pos: Position,
}

impl<R> Reader<R> {
    pub fn new(input: R) -> Self {
        Reader {
            input,

            buffer: [0; 4],
            buffer_i: 0,

            next_pos: Position {
                column: 0,
                line: 0,
                index: 0,
            },
        }
    }
}

impl<R> pipeline::Stage for Reader<R>
where
    R: io::Read,
{
    type Item = Char;
    type Error = Error;

    fn next(&mut self) -> Result<Self::Item, Self::Error> {
        loop {
            if self.buffer_i >= self.buffer.len() {
                // This can only happen if an error occurred before.
                return Err(Error::EndOfStream);
            }

            let result = self
                .input
                .read_exact(&mut self.buffer[self.buffer_i..=self.buffer_i]);
            self.buffer_i += 1;

            match result {
                Ok(()) => (),
                Err(error) => match error.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        return Err(Error::EndOfStream);
                    }
                    _ => {
                        return Err(Error::Io(error));
                    }
                },
            }

            match str::from_utf8(&self.buffer[..self.buffer_i]) {
                Ok(s) => {
                    // Unless there's a bug in this method that causes multiple
                    // good characters to be in the buffer at once, this should
                    // never panic.
                    assert_eq!(s.chars().count(), 1);

                    // Can't panic. We just asserted that there is exactly one
                    // char.
                    let c = s.chars().next().unwrap();

                    let c = Char {
                        c,
                        pos: self.next_pos,
                    };

                    self.next_pos.column += 1;
                    if c.c == '\n' {
                        self.next_pos.column = 0;
                        self.next_pos.line += 1;
                    }

                    self.next_pos.index += self.buffer_i;
                    self.buffer_i = 0;

                    return Ok(c);
                }
                Err(error) => match self.buffer_i {
                    i if i == 4 => {
                        return Err(Error::Utf8(error));
                    }
                    i if i < 4 => {
                        continue;
                    }
                    _ => {
                        unreachable!();
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    Io(io::Error),
    Utf8(Utf8Error),
}
