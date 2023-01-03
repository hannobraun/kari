use std::{
    collections::HashMap,
    fmt,
    io::{self, SeekFrom},
    iter::repeat,
    str::from_utf8,
};

use termion::{color, style};

use crate::{
    call_stack::CallStack,
    context,
    interpreter::stream::Stream,
    pipeline::{parser, tokenizer::Span},
};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub call_stack: CallStack,
}

impl Error {
    pub fn print(
        &self,
        streams: &mut HashMap<String, Box<dyn Stream>>,
        stderr: &mut dyn io::Write,
    ) -> io::Result<()> {
        write!(
            stderr,
            "\n{}{}ERROR:{} {}{}\n",
            color::Fg(color::Red),
            style::Bold,
            color::Fg(color::Reset),
            self,
            style::Reset,
        )?;

        let mut sources = Vec::new();
        self.kind.sources(&mut sources);

        for source in sources {
            print_source(source, streams, stderr)?;
        }

        self.kind.write_hint(stderr)?;

        for stack_frame in self.call_stack.frames.iter().rev() {
            write!(
                stderr,
                "\n{}Called by:{}\n",
                color::Fg(color::Cyan),
                color::Fg(color::Reset),
            )?;
            match &stack_frame.src {
                None => {
                    panic!("Tried to format a null source");
                }
                Some(src) => {
                    print_source(src, streams, stderr)?;
                }
            }
        }

        writeln!(stderr)?;

        Ok(())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Context(error) => error.fmt(f),
            ErrorKind::Parser(error) => error.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Context(context::Error),
    Parser(parser::Error),
}

impl ErrorKind {
    pub fn sources<'r>(&'r self, sources: &mut Vec<&'r Span>) {
        match self {
            ErrorKind::Context(error) => error.sources(sources),
            ErrorKind::Parser(error) => error.sources(sources),
        }
    }

    pub fn write_hint(&self, stderr: &mut dyn io::Write) -> io::Result<()> {
        match self {
            ErrorKind::Context(error) => error.write_hint(stderr),
            ErrorKind::Parser(_) => Ok(()),
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

fn print_source<Stream>(
    src: &Span,
    streams: &mut HashMap<String, Stream>,
    stderr: &mut dyn io::Write,
) -> io::Result<()>
where
    Stream: io::Read + io::Seek,
{
    let stream = streams.get_mut(&src.stream_name).unwrap();

    let start = search_backward(src.start.index, stream)?;
    let end = search_forward(src.end.index, stream)?;

    let mut buffer = repeat(0).take(end - start).collect::<Vec<_>>();
    stream.seek(SeekFrom::Start(start as u64))?;
    stream.read_exact(&mut buffer)?;

    // Can't fail. If this wasn't UTF-8, we never would have gotten to the point
    // where we need to render a span.
    let buffer = from_utf8(&buffer).unwrap();

    writeln!(
        stderr,
        "  {}=> {}{}:{}:{}{}",
        color::Fg(color::Magenta),
        color::Fg(color::LightBlue),
        src.stream_name,
        src.start.line + 1,
        src.start.column + 1,
        color::Fg(color::Reset),
    )?;
    writeln!(stderr)?;

    for (i, line) in buffer.lines().enumerate() {
        let line_number = src.start.line + i;
        let line_len = line.chars().count();

        writeln!(
            stderr,
            "{}{:5} {}| {}{}{}{}{}",
            color::Fg(color::LightBlue),
            line_number + 1,
            color::Fg(color::Magenta),
            style::Bold,
            color::Fg(color::LightWhite),
            line.replace('\t', "    "),
            color::Fg(color::Reset),
            style::Reset,
        )?;

        let start_column = if line_number == src.start.line {
            src.start.column
        } else {
            0
        };
        let end_column = if line_number == src.end.line {
            src.end.column + 1
        } else {
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
        for column in 0..line_len {
            if column >= start_column && column < end_column {
                write!(stderr, "^")?;
            } else if line.chars().nth(column) == Some('\t') {
                // Before we printed the line above, we replaced each tab with 4
                // spaces. This means, if we encounter a tab here, we know that
                // we can just replace it with 4 spaces to make everything line
                // up.
                write!(stderr, "    ")?;
            } else {
                write!(stderr, " ")?;
            }
        }
        writeln!(stderr, "{}{}", style::Reset, color::Fg(color::Reset),)?;
    }

    Ok(())
}

fn search_backward<Stream>(
    from: usize,
    stream: &mut Stream,
) -> io::Result<usize>
where
    Stream: io::Read + io::Seek,
{
    stream.seek(SeekFrom::Start(from as u64 + 1))?;

    while stream.stream_position()? > 1 {
        stream.seek(SeekFrom::Current(-2))?;

        let mut buffer = [0];
        stream.read_exact(&mut buffer)?;

        if buffer[0] == b'\n' {
            let pos = stream.stream_position()?;
            return Ok(pos as usize);
        }
    }

    Ok(0)
}

fn search_forward<Stream>(from: usize, stream: &mut Stream) -> io::Result<usize>
where
    Stream: io::Read + io::Seek,
{
    stream.seek(SeekFrom::Start(from as u64))?;

    loop {
        let pos = stream.stream_position()?;

        let mut buffer = [0];
        match stream.read(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b'\n' {
                    let pos = stream.stream_position()?;
                    return Ok(pos as usize);
                }
            }
            Err(error) => {
                if let io::ErrorKind::UnexpectedEof = error.kind() {
                    return Ok(pos as usize);
                } else {
                    return Err(error);
                }
            }
        }
    }
}
