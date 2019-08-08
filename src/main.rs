mod core {
    pub mod context;
    pub mod expression;
    pub mod functions;
    pub mod span;
    pub mod stack;
}

mod builtins;
mod evaluator;
mod interpreter;
mod pipeline;


use std::{
    fs::File,
    io::stdin,
    process::exit,
};

use acc_reader::AccReader;
use clap::{
    App,
    Arg,
};
use walkdir::WalkDir;


fn main() {
    let args = App::new("Kari")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Interpreter for the Kari prorgamming language")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .index(1)
                .help("The program to execute, without the \".kr\" extension.")
        )
        .arg(
            Arg::with_name("test")
                .long("test")
                .short("t")
                .help("Run a test if PATH is specified, all of them otherwise")
        )
        .get_matches();

    let kind = if args.is_present("test") {
        ProgramKind::Test
    } else {
        ProgramKind::Regular
    };

    match args.value_of("path") {
        Some(name) => {
            let path = format!("kr/{}/{}.kr", kind.base(), name);
            run_program(&path);
        }
        None => {
            match kind {
                ProgramKind::Regular => {
                    interpreter::run(
                        "<stdin>",
                        AccReader::new(stdin().lock()),
                    );
                }
                ProgramKind::Test => {
                    print!("\n");

                    for result in WalkDir::new("kr/tests") {
                        let entry = match result {
                            Ok(entry) => {
                                entry
                            }
                            Err(error) => {
                                print!(
                                    "ERROR: Error walking tests directory: {}",
                                    error,
                                );
                                if let Some(path) = error.path() {
                                    print!(" ({})\n", path.display());
                                }
                                exit(1);
                            }
                        };

                        let path = entry.path();
                        let path = match path.to_str() {
                            Some(path) => {
                                path
                            }
                            None => {
                                print!(
                                    "ERROR: Cannot conver path to UTF-8: {}\n",
                                    path.to_string_lossy(),
                                );
                                exit(1);
                            }
                        };

                        if !path.ends_with(".kr") {
                            continue;
                        }

                        let success = run_program(path);
                        if success {
                            print!("    OK {}\n", path);
                        }
                    }

                    print!("\n");
                }
            }
        }
    }
}


fn run_program(path: &str) -> bool {
    let file = match File::open(path) {
        Ok(file) => {
            file
        }
        Err(error) => {
            print!(
                "\nERROR: Failed to open file {} ({})\n\n",
                path,
                error,
            );
            exit(1);
        }
    };

    interpreter::run(path, file)
}


enum ProgramKind {
    Regular,
    Test,
}

impl ProgramKind {
    fn base(&self) -> &'static str {
        match self {
            ProgramKind::Regular => "examples",
            ProgramKind::Test    => "tests",
        }
    }
}
