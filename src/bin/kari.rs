use std::{
    fs::File,
    io::{stderr, stdin, stdout},
    process::exit,
};

use acc_reader::AccReader;
use structopt::StructOpt;

use kari::Interpreter;

#[derive(StructOpt)]
struct Options {
    path: Option<String>,
}

fn main() {
    let stdout = Box::new(stdout());
    let stderr = Box::new(stderr());

    let options = Options::from_args();

    match options.path {
        Some(path) => {
            let file =
                File::open(&path).unwrap_or_else(|error| {
                    print!(
                        "\nERROR: Failed to open file {} ({})\n\n",
                        path, error,
                    );
                    exit(1);
                });

            let _ = Interpreter::new(stdout, stderr)
                .with_default_builtins()
                .with_default_prelude(&mut ())
                .unwrap_or_else(|error| {
                    println!("ERROR: Failed to load prelude: {}", error);
                    exit(1);
                })
                .with_default_modules()
                .run(&mut (), path.into(), Box::new(file));
        }
        None => {
            let stdin = Box::new(AccReader::new(stdin()));

            let _ = Interpreter::new(stdout, stderr)
                .with_default_builtins()
                .with_default_prelude(&mut ())
                .unwrap_or_else(|error| {
                    println!("ERROR: Failed to load prelude: {}", error);
                    exit(1);
                })
                .with_default_modules()
                .run(&mut (), "<stdin>".into(), stdin);
        }
    }
}
