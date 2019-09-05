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

use termion::{
    color,
    style,
};

use crate::{
    context,
    data::span::Span,
    interpreter::stream::Stream,
    pipeline::parser,
};


#[derive(Debug)]
pub struct Error {
    pub kind:        ErrorKind,
    pub stack_trace: Vec<Span>,
}

impl Error {
    pub fn print(self,
        streams: &mut HashMap<String, Box<dyn Stream>>,
        stderr:  &mut dyn io::Write,
    )
        -> io::Result<()>
    {
        write!(
            stderr,
            "\n{}{}ERROR:{} {}{}\n",
            color::Fg(color::Red), style::Bold,
            color::Fg(color::Reset),
            self,
            style::Reset,
        )?;

        let mut spans = Vec::new();
        self.kind.spans(&mut spans);

        for span in spans {
            print_span(
                &span,
                streams,
                stderr,
            )?;
        }

        for span in self.stack_trace.into_iter().rev() {
            write!(
                stderr,
                "\n{}Called by:{}\n",
                color::Fg(color::Cyan),
                color::Fg(color::Reset),
            )?;
            print_span(
                &span,
                streams,
                stderr,
            )?;
        }

        write!(stderr, "\n")?;

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


#[derive(Debug)]
pub enum ErrorKind {
    Context(context::Error),
    Parser(parser::Error),
}

impl ErrorKind {
    pub fn spans(self, spans: &mut Vec<Span>) {
        match self {
            ErrorKind::Context(error) => error.spans(spans),
            ErrorKind::Parser(error)  => error.spans(spans),
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
    span:    &Span,
    streams: &mut HashMap<String, Stream>,
    stderr:  &mut dyn io::Write,
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

    write!(
        stderr,
        "  {}=> {}{}:{}:{}{}\n",
        color::Fg(color::Magenta),

        color::Fg(color::LightBlue),
        span.stream,
        span.start.line + 1,
        span.start.column + 1,
        color::Fg(color::Reset),
    )?;
    write!(stderr, "\n")?;

    for (i, line) in buffer.lines().enumerate() {
        let line_number = span.start.line + i;
        let line_len    = line.chars().count();

        write!(
            stderr,
            "{}{:5} {}| {}{}{}{}{}\n",
            color::Fg(color::LightBlue),
            line_number + 1,

            color::Fg(color::Magenta),

            style::Bold, color::Fg(color::LightWhite),
            line.replace("\t", "    "),
            color::Fg(color::Reset), style::Reset,
        )?;

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

        write!(
            stderr,
            "        {}{}",
            color::Fg(color::LightRed),
            style::Bold,
        )?;
        for column in 0 .. line_len {
            if column >= start_column && column <  end_column {
                write!(stderr, "^")?;
            }
            else {
                if line.chars().nth(column) == Some('\t') {
                    // Before we printed the line above, we replaced each tab
                    // with 4 spaces. This means, if we encounter a tab here, we
                    // know that we can just replace it with 4 spaces to make
                    // everything line up.
                    write!(stderr, "    ")?;
                }
                else {
                    write!(stderr, " ")?;
                }
            }
        }
        write!(
            stderr,
            "{}{}\n",
            style::Reset,
            color::Fg(color::Reset),
        )?;
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
