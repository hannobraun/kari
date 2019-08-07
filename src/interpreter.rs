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
        stream::Stream,
        tokenizer::Tokenizer,
    },
};


pub fn run<Program>(name: &str, program: Program) -> bool
    where Program: io::Read
{
    let     reader    = Reader::new(program);
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


fn print_error<Program>(
    error:    Error,
    program:  &str,
    recorder: &mut Recorder<Reader<Program>>,
)
    where Program: io::Read
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
            program,
            &mut recorder.chars().clone(),
        );
    }

    for span in error.stack_trace.into_iter().rev() {
        print!("\n\nCalled by:\n");
        print_span(
            span,
            program,
            &mut recorder.chars().clone(),
        );
    }

    print!("\n");
}

fn print_span(span: Span, program: &str, chars: &mut Vec<Char>) {
    chars.retain(|c|
        c.pos.line >= span.start.line
            && c.pos.line <= span.end.line
    );

    print!(
        "  => {}:{}:{}\n",
        program,
        span.start.line + 1,
        span.start.column + 1,
    );
    print!("      |\n");

    // This makes heavy assumptions about the structure of `chars`,
    // namely that chars' position's are consecutive, that chars in the
    // same line actually have the same line recorded in their position,
    // stuff like that.
    for line in chars.split(|c| c.c == '\n') {
        let first = match line.first() {
            Some(first) => first.pos,
            None        => continue,
        };
        let last = match line.last() {
            Some(last) => last.pos,
            None       => continue,
        };

        let start_column = if first.line == span.start.line {
            span.start.column
        }
        else {
            0
        };
        let end_column = if first.line == span.end.line {
            span.end.column
        }
        else {
            last.column
        };

        print!("{:5} | ", first.line + 1);
        for c in line {
            print!("{}", c.c);
        }
        print!("\n");

        print!("      | ");
        for column in 0 ..= last.column {
            if column >= start_column && column <= end_column {
                print!("^");
            }
            else {
                print!(" ");
            }
        }
    }
}
