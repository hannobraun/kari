use std::io;

use crate::{
    evaluator::Evaluator,
    parser::Parser,
    reader::Reader,
    recorder::Recorder,
    stream::Stream,
    tokenizer::Tokenizer,
};


pub fn run<Program>(name: &str, program: Program)
    where Program: io::Read
{
    let     reader    = Reader::new(program);
    let mut recorder  = Recorder::new(reader);
    let     tokenizer = Tokenizer::new(&mut recorder);
    let     parser    = Parser::new(tokenizer);
    let     evaluator = Evaluator::new();

    if let Err(error) = evaluator.run(parser) {
        print!("\nERROR: {}\n", error);

        if let Some(span) = error.span() {
            // Read the rest of the line, so the error isn't cut off.
            while let Ok(c) = recorder.next() {
                if c == '\n' {
                    break;
                }
            }

            let mut chars = recorder.chars();
            chars.retain(|c|

                c.pos.line >= span.start.line
                    && c.pos.line <= span.end.line
            );

            print!("  => {}:{}:{}\n", name, span.start.line, span.start.column);
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

        print!("\n");
    }
}
