use std::{
    collections::HashMap,
    io::{
        self,
        SeekFrom,
    },
    fmt,
    iter::repeat,
    str::from_utf8,
};

use crate::{
    builtins::context,
    data::span::Span,
    interpreter::stream::Stream,
    pipeline::parser,
};


pub struct Error {
    pub kind:        ErrorKind,
    pub stack_trace: Vec<Span>,
}

impl Error {
    pub fn print(self,
        streams: &mut HashMap<String, Box<Stream>>,
    )
        -> io::Result<()>
    {
        print!("\nERROR: {}\n", self);

        if let Some(span) = self.kind.span() {
            print_span(
                span,
                streams,
            )?;
        }

        for span in self.stack_trace.into_iter().rev() {
            print!("\nCalled by:\n");
            print_span(
                span,
                streams,
            )?;
        }

        print!("\n");

        Ok(())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Context(error) => error.fmt(f),
            ErrorKind::Parser(error)  => error.fmt(f),
        }
    }
}


pub enum ErrorKind {
    Context(context::Error),
    Parser(parser::Error),
}

impl ErrorKind {
    pub fn span(self) -> Option<Span> {
        match self {
            ErrorKind::Context(error) => error.span(),
            ErrorKind::Parser(error)  => error.span(),
        }
    }
}

impl From<context::Error> for ErrorKind {
    fn from(from: context::Error) -> Self {
        ErrorKind::Context(from)
    }
}

impl From<parser::Error> for ErrorKind {
    fn from(from: parser::Error) -> Self {
        ErrorKind::Parser(from)
    }
}


fn print_span<Stream>(
    span:    Span,
    streams: &mut HashMap<String, Stream>,
)
    -> io::Result<()>
    where Stream: io::Read + io::Seek
{
    let stream = streams
        .get_mut(&span.stream)
        .unwrap();

    let start = search_backward(span.start.index, stream)?;
    let end   = search_forward(span.end.index, stream)?;

    let mut buffer = repeat(0).take(end - start).collect::<Vec<_>>();
    stream.seek(SeekFrom::Start(start as u64))?;
    stream.read_exact(&mut buffer)?;

    // Can't fail. If this wasn't UTF-8, we never would have gotten to the point
    // where we need to render a span.
    let buffer = from_utf8(&buffer).unwrap();

    print!(
        "  => {}:{}:{}\n",
        span.stream,
        span.start.line + 1,
        span.start.column + 1,
    );
    print!("\n");

    for (i, line) in buffer.lines().enumerate() {
        let line_number = span.start.line + i;
        let line_len    = line.chars().count();

        print!("{:5} | {}\n", line_number + 1, line);

        let start_column = if line_number == span.start.line {
            span.start.column
        }
        else {
            0
        };
        let end_column = if line_number == span.end.line {
            span.end.column + 1
        }
        else {
            line_len
        };

        if start_column == end_column {
            continue;
        }

        print!("        ");
        for column in 0 .. line_len {
            if column >= start_column && column <  end_column {
                print!("^");
            }
            else {
                print!(" ");
            }
        }

        print!("\n");
    }

    Ok(())
}


fn search_backward<Stream>(from: usize, stream: &mut Stream)
    -> io::Result<usize>
    where Stream: io::Read + io::Seek
{
    stream.seek(SeekFrom::Start(from as u64 + 1))?;

    while stream.seek(SeekFrom::Current(0))? > 1 {
        stream.seek(SeekFrom::Current(-2))?;

        let mut buffer = [0];
        stream.read(&mut buffer)?;

        if buffer[0] == b'\n' {
            let pos = stream.seek(SeekFrom::Current(0))?;
            return Ok(pos as usize);
        }
    }

    Ok(0)
}

fn search_forward<Stream>(from: usize, stream: &mut Stream)
    -> io::Result<usize>
    where Stream: io::Read + io::Seek
{
    stream.seek(SeekFrom::Start(from as u64))?;

    loop {
        let pos = stream.seek(SeekFrom::Current(0))?;

        let mut buffer = [0];
        match stream.read(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b'\n' {
                    let pos = stream.seek(SeekFrom::Current(0))?;
                    return Ok(pos as usize);
                }
            }
            Err(error) => {
                if let io::ErrorKind::UnexpectedEof = error.kind() {
                    return Ok(pos as usize);
                }
                else {
                    return Err(error);
                }
            }
        }
    }
}
