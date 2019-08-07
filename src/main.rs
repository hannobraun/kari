mod core {
    pub mod builtins;
    pub mod context;
    pub mod expression;
    pub mod functions;
    pub mod span;
    pub mod stack;
}
mod pipeline {
    pub mod evaluator;
    pub mod parser;
    pub mod reader;
    pub mod recorder;
    pub mod stream;
    pub mod tokenizer;
}

mod interpreter;


use std::{
    fs::File,
    io,
};

use clap::{
    App,
    Arg,
};


fn main() {
    let args = App::new("Kari")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Hanno Braun <hb@hannobraun.de>")
        .about("Interpreter for the Kari prorgamming language")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .index(1)
                .help("The program to execute, without the \".kr\" extension.")
        )
        .arg(
            Arg::with_name("test")
                .long("test")
                .short("t")
                .help("Run a test if PATH is specified, all of them otherwise")
        )
        .get_matches();

    let kind = if args.is_present("test") {
        ProgramKind::Test
    } else {
        ProgramKind::Regular
    };

    match args.value_of("path") {
        Some(name) => {
            run_program(kind, name);
        }
        None => {
            interpreter::run("<stdin>", io::stdin().lock())
        }
    }
}


fn run_program(kind: ProgramKind, name: &str) {
    let path = format!("kr/{}/{}.kr", kind.base(), name);
    let file = match File::open(&path) {
        Ok(file) => {
            file
        }
        Err(error) => {
            print!(
                "\nERROR: Failed to open file {} ({})\n\n",
                path,
                error,
            );
            return;
        }
    };

    interpreter::run(&path, file)
}


enum ProgramKind {
    Regular,
    Test,
}

impl ProgramKind {
    fn base(&self) -> &'static str {
        match self {
            ProgramKind::Regular => "examples",
            ProgramKind::Test    => "tests",
        }
    }
}
