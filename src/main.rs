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

    runner::run(program);
}
