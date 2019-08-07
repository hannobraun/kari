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
        .get_matches();

    match args.value_of("path") {
        Some(name) => {
            let path = format!("kr/examples/{}.kr", name);
            let file = match File::open(&path) {
                Ok(file) => {
                    file
                }
                Err(error) => {
                    print!(
                        "\nERROR: Failed to open file file {} ({})\n\n",
                        path,
                        error,
                    );
                    return;
                }
            };

            interpreter::run(&path, file)
        }
        None => {
            interpreter::run("stdin", io::stdin().lock())
        }
    }
}
