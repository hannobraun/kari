use std::{
    fs::File,
    io::{
        stderr,
        stdout,
    },
    process::exit,
};

use termion::{
    color,
    style,
};
use walkdir::WalkDir;

use kari::{
    builtins,
    data::functions::Functions,
    interpreter::evaluator::Evaluator,
};


fn main() {
    print!("\n");

    for result in WalkDir::new("kr/tests") {
        let entry = result.unwrap_or_else(|error| {
            print!(
                "ERROR: Error walking tests directory: {}",
                error,
            );
            if let Some(path) = error.path() {
                print!(" ({})\n", path.display());
            }
            exit(1);
        });

        let path = entry.path();
        let path = path.to_str()
            .unwrap_or_else(|| {
                print!(
                    "ERROR: Cannot convert path to UTF-8: {}\n",
                    path.to_string_lossy(),
                );
                exit(1);
            });

        if !path.ends_with(".kr") {
            continue;
        }

        let file = File::open(path)
            .unwrap_or_else(|error| {
                print!(
                    "\nERROR: Failed to open file {} ({})\n\n",
                    path,
                    error,
                );
                exit(1);
            });

        let stdout = Box::new(stdout());
        let stderr = Box::new(stderr());

        let mut functions = Functions::new();
        builtins::builtins(&mut functions);

        let prelude = kari::prelude()
            .unwrap_or_else(|error| {
                print!("ERROR: Failed to load prelude: {}\n", error);
                exit(1);
            });

        let success = Evaluator::new(stdout, stderr, (), functions)
            .run(path.into(), prelude, Box::new(file));

        if success {
            print!("    {}{}OK{}{} {}\n",
                style::Bold, color::Fg(color::LightGreen),
                color::Fg(color::Reset), style::Reset,
                path,
            );
        }
    }

    print!("\n");
}
