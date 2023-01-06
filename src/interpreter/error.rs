use std::{collections::HashMap, fmt, io};

use termion::{color, style};

use crate::{
    call_stack::CallStack, context, interpreter::stream::Stream,
    pipeline::parser, source::Span,
};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub call_stack: CallStack,
}

impl Error {
    pub fn print(
        &self,
        _: &mut HashMap<String, Box<dyn Stream>>,
        sources: &HashMap<String, String>,
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

        let mut spans = Vec::new();
        self.kind.spans(&mut spans);

        for span in spans {
            print_source(span, sources, stderr)?;
        }

        self.kind.write_hint(stderr)?;

        for stack_frame in self.call_stack.frames.iter().rev() {
            write!(
                stderr,
                "\n{}Called by:{}\n",
                color::Fg(color::Cyan),
                color::Fg(color::Reset),
            )?;
            match &stack_frame.span {
                None => {
                    panic!("Tried to format a null source");
                }
                Some(src) => {
                    print_source(src, sources, stderr)?;
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
    pub fn spans<'r>(&'r self, spans: &mut Vec<&'r Span>) {
        match self {
            ErrorKind::Context(error) => error.spans(spans),
            ErrorKind::Parser(error) => error.spans(spans),
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

fn print_source(
    span: &Span,
    sources: &HashMap<String, String>,
    stderr: &mut dyn io::Write,
) -> io::Result<()> {
    let source = sources.get(&span.stream_name).unwrap();

    let start = source[..span.start.index].rfind('\n').unwrap_or(0);
    let end = span.end.index
        + source[span.end.index..]
            .find('\n')
            .unwrap_or(source.len() - 1)
        + 1;

    let source = &source[start..end];

    writeln!(
        stderr,
        "  {}=> {}{}:{}:{}{}",
        color::Fg(color::Magenta),
        color::Fg(color::LightBlue),
        span.stream_name,
        span.start.line + 1,
        span.start.column + 1,
        color::Fg(color::Reset),
    )?;
    writeln!(stderr)?;

    for (i, line) in source.lines().enumerate() {
        let line_number = span.start.line + i;
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

        let start_column = if line_number == span.start.line {
            span.start.column
        } else {
            0
        };
        let end_column = if line_number == span.end.line {
            span.end.column + 1
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
