use std::{
    io::{
        self,
        SeekFrom,
    },
    iter::repeat,
    str::from_utf8,
};

use crate::{
    core::span::Span,
    evaluator::{
        Error,
        Evaluator,
    },
    pipeline,
};


pub fn run<Stream>(name: &str, mut stream: Stream) -> bool
    where Stream: io::Read + io::Seek
{
    if let Err(error) = Evaluator::run(pipeline::new(stream.by_ref())) {
        if let Err(error) = print_error(error, name, stream) {
            print!("Error printing error: {}\n", error)
        }
        return false;
    }

    true
}


fn print_error<Stream>(
        error:  Error,
        name:   &str,
    mut stream: Stream,
)
    -> io::Result<()>
    where Stream: io::Read + io::Seek
{
    print!("\nERROR: {}\n", error);

    if let Some(span) = error.span() {
        print_span(
            span,
            name,
            &mut stream,
        )?;
    }

    for span in error.stack_trace.into_iter().rev() {
        print!("\nCalled by:\n");
        print_span(
            span,
            name,
            &mut stream,
        )?;
    }

    print!("\n");

    Ok(())
}

fn print_span<Stream>(
    span:   Span,
    name:   &str,
    stream: &mut Stream,
)
    -> io::Result<()>
    where Stream: io::Read + io::Seek
{
    let start = search_backward(span.start.index, stream)?;
    let end   = search_forward(span.end.index, stream)?;

    let mut buffer = repeat(0).take(end - start).collect::<Vec<_>>();
    stream.seek(SeekFrom::Start(start as u64))?;
    stream.read_exact(&mut buffer)?;

    // Can't fail. If this weren't UTF-8, we never would have gotten to the
    // stage where we need to render a span.
    let buffer = from_utf8(&buffer).unwrap();

    print!(
        "  => {}:{}:{}\n",
        name,
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
