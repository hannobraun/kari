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
            let name = env::args().skip(1).next().unwrap();
            let path = format!("examples/{}.kr", name);
            let file = File::open(path).unwrap();

            runner::run(file)
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
