use std::io;

use crate::{
    core::span::Span,
    pipeline::{
        evaluator::{
            Error,
            Evaluator,
        },
        parser::Parser,
        reader::{
            Char,
            Reader,
        },
        recorder::Recorder,
        tokenizer::Tokenizer,
        Stage as _,
    },
};


pub fn run<Stream>(name: &str, stream: Stream) -> bool
    where Stream: io::Read
{
    let     reader    = Reader::new(stream);
    let mut recorder  = Recorder::new(reader);
    let     tokenizer = Tokenizer::new(&mut recorder);
    let     parser    = Parser::new(tokenizer);
    let     evaluator = Evaluator::new();

    if let Err(error) = evaluator.run(parser) {
        print_error(error, name, &mut recorder);
        return false;
    }

    true
}


fn print_error<Stream>(
    error:    Error,
    name:     &str,
    recorder: &mut Recorder<Reader<Stream>>,
)
    where Stream: io::Read
{
    // Read the rest of the line, so the error isn't cut off.
    while let Ok(c) = recorder.next() {
        if c == '\n' {
            break;
        }
    }

    print!("\nERROR: {}\n", error);

    if let Some(span) = error.span() {
        print_span(
            span,
            name,
            &mut recorder.chars().clone(),
        );
    }

    for span in error.stack_trace.into_iter().rev() {
        print!("\n\nCalled by:\n");
        print_span(
            span,
            name,
            &mut recorder.chars().clone(),
        );
    }

    print!("\n");
}

fn print_span(
    span:  Span,
    name:  &str,
    chars: &mut Vec<Char>,
) {
    chars.retain(|c|
        c.pos.line >= span.start.line
            && c.pos.line <= span.end.line
    );

    print!(
        "  => {}:{}:{}\n",
        name,
        span.start.line + 1,
        span.start.column + 1,
    );
    print!("\n");

    // This makes heavy assumptions about the structure of `chars`,
    // namely that chars' position's are consecutive, that chars in the
    // same line actually have the same line recorded in their position,
    // stuff like that.
    for (i, line) in chars.split(|c| c.c == '\n').enumerate() {
        let line_number = span.start.line + i;
        let line_len    = line.len();

        if line_len == 0 {
            continue;
        }

        print!("{:5} | ", line_number + 1);
        for c in line {
            print!("{}", c.c);
        }
        print!("\n");

        let start_column = if line_number == span.start.line {
            span.start.column
        }
        else {
            0
        };
        let end_column = if line_number == span.end.line {
            span.end.column
        }
        else {
            line_len
        };

        print!("        ");
        for column in 0 .. line_len {
            if column >= start_column && column <= end_column {
                print!("^");
            }
            else {
                print!(" ");
            }
        }

        print!("\n");
    }
}
