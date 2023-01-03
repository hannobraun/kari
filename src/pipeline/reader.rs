use crate::source::{Char, Position};

use std::{
    io,
    str::{self, Utf8Error},
};

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

impl<R> Reader<R>
where
    R: io::Read,
{
    pub fn next_char(&mut self, source: &mut String) -> Result<Char, Error> {
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
                    let mut chars = s.chars();

                    // This shouldn't panic. We just got an `Ok(...)` from the
                    // parsing method, so there should be a char in there.
                    let c = chars.next().unwrap();

                    // Unless there's a bug in this method that causes multiple
                    // good characters to be in the buffer at once, this should
                    // never panic.
                    assert!(chars.next().is_none());

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

                    assert_eq!(source.len(), c.pos.index);
                    source.push(c.c);

                    return Ok(c);
                }
                Err(error) => match self.buffer_i {
                    i if i < 4 => {
                        // We haven't collected 4 bytes yet, so the next byte
                        // might give us a valid character.
                        continue;
                    }
                    _ => {
                        return Err(Error::Utf8(error));
                    }
                },
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("End of stream")]
    EndOfStream,

    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("UTF-8 error")]
    Utf8(#[from] Utf8Error),
}
