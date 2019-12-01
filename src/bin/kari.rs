use std::{
    fs::File,
    io::{
        stderr,
        stdin,
        stdout,
    },
    process::exit,
};

use acc_reader::AccReader;
use clap::{
    App,
    Arg,
};

use kari::Interpreter;


fn main() {
    let args = App::new("Kari")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Interpreter for the Kari programming language")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .index(1)
                .help("The program to execute, without the \".kr\" extension.")
        )
        .get_matches();

    let stdout = Box::new(stdout());
    let stderr = Box::new(stderr());

    match args.value_of("path") {
        Some(path) => {
            let file = File::open(&path)
                .unwrap_or_else(|error| {
                    print!(
                        "\nERROR: Failed to open file {} ({})\n\n",
                        path,
                        error,
                    );
                    exit(1);
                });

            let _ = Interpreter::new(stdout, stderr)
                .with_default_builtins()
                .with_default_prelude(&mut ())
                .unwrap_or_else(|error| {
                    print!("ERROR: Failed to load prelude: {}\n", error);
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
                    print!("ERROR: Failed to load prelude: {}\n", error);
                    exit(1);
                })
                .with_default_modules()
                .run(&mut (), "<stdin>".into(), stdin);
        }
    }
}
