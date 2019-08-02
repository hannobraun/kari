mod interpreter;
mod reader;
mod runner;
mod tokenizer;


use std::{
    env,
    fs::File,
};


fn main() {
    let program = env::args().skip(1).next().unwrap();
    let file    = format!("examples/{}.kr", program);
    let program = File::open(file).unwrap();

    if let Err(error) = runner::run(program) {
        match error {
            runner::Error::Reader(error)      => print!("{:?}", error),
            runner::Error::Interpreter(error) => print!("{}", error),
        }
    }
}
