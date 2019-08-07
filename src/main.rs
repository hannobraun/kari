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
    env,
    fs::File,
    io,
};


fn main() {
    match env::args().count() {
        1 => {
            interpreter::run("stdin", io::stdin().lock())
        },
        2 => {
            // Can't panic, as we just verified that there are two arguments.
            let name = env::args().skip(1).next().unwrap();

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
        _ => {
            print!("\nERROR: Expecting zero or one arguments\n\n");
            return;
        }
    }
}
