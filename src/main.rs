mod functions;
mod interpreter;
mod reader;
mod runner;
mod stack;
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

            runner::run(file)
        }
        _ => {
            print!("\nERROR: Expecting zero or one arguments\n\n");
            return;
        }
    };

    if let Err(error) = result {
        print!("\nERROR: {:?}\n\n", error);
    }
}
