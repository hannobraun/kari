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

use kari::{
    extensions::Extensions,
    interpreter::evaluator::Evaluator,
};


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

    match args.value_of("path") {
        Some(name) => {
            let path = format!("kr/examples/{}.kr", name);

            let file = File::open(&path)
                .unwrap_or_else(|error| {
                    print!(
                        "\nERROR: Failed to open file {} ({})\n\n",
                        path,
                        error,
                    );
                    exit(1);
                });

            let stdout     = Box::new(stdout());
            let stderr     = Box::new(stderr());
            let extensions = Extensions::none();

            Evaluator::new(stdout, stderr, extensions)
                .run(path.into(), Box::new(file));
        }
        None => {
            let stdout     = Box::new(stdout());
            let stderr     = Box::new(stderr());
            let extensions = Extensions::none();

            Evaluator::new(stdout, stderr, extensions)
                .run("<stdin>".into(), Box::new(AccReader::new(stdin())));
        }
    }
}
