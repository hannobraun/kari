mod functions;
mod interpreter;
mod reader;
mod runner;
mod tokenizer;


use std::{
    env,
    fs::File,
    io,
};


fn main() {
    let result = match env::args().count() {
        1 => {
            runner::run(io::stdin().lock())
        },
        2 => {
            let program = env::args().skip(1).next().unwrap();
            let file    = format!("examples/{}.kr", program);
            let program = File::open(file).unwrap();

            runner::run(program)
        }
        _ => {
            print!("ERROR: Expecting zero or one arguments");
            return;
        }
    };

    if let Err(error) = result {
        match error {
            runner::Error::Reader(error)      => print!("{:?}", error),
            runner::Error::Interpreter(error) => print!("{}", error),
        }
    }
}
