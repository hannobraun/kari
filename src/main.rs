mod builtins;
mod context;
mod evaluator;
mod expression;
mod functions;
mod parser;
mod reader;
mod recorder;
mod runner;
mod stack;
mod stream;
mod tokenizer;


use std::{
    env,
    fs::File,
    io,
};


fn main() {
    match env::args().count() {
        1 => {
            runner::run("stdin", io::stdin().lock())
        },
        2 => {
            // Can't panic, as we just verified that there are two arguments.
            let name = env::args().skip(1).next().unwrap();

            let path = format!("examples/{}.kr", name);
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

            runner::run(&path, file)
        }
        _ => {
            print!("\nERROR: Expecting zero or one arguments\n\n");
            return;
        }
    }
}
