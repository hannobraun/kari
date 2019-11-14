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

use kari::interpreter::evaluator::Evaluator;


fn main() {
    print!("\n");

    let mut results = Vec::new();

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

        let success = Evaluator::new(stdout, stderr)
            .with_default_builtins()
            .with_default_prelude(&mut ())
            .unwrap_or_else(|error| {
                print!("ERROR: Failed to load prelude: {}\n", error);
                exit(1);
            })
            .with_default_modules()
            .run(&mut (), path.into(), Box::new(file));

        results.push((success, path.to_owned()));
    }

    for (success, path) in results {
        if success.is_ok() {
            print!("       {}{}OK{}{} {}\n",
                style::Bold, color::Fg(color::LightGreen),
                color::Fg(color::Reset), style::Reset,
                path,
            );
        }
        else {
            print!("    {}{}ERROR{}{} {}\n",
                style::Bold, color::Fg(color::Red),
                color::Fg(color::Reset), style::Reset,
                path,
            );
        }
    }

    print!("\n");
}
