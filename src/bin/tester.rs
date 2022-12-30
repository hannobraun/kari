use std::{
    fs::File,
    io::{stderr, stdout},
    process::exit,
};

use termion::{color, style};
use walkdir::WalkDir;

use kari::Interpreter;

fn main() {
    println!();

    let mut results = Vec::new();

    for result in WalkDir::new("kr/tests") {
        let entry = result.unwrap_or_else(|error| {
            print!("ERROR: Error walking tests directory: {}", error,);
            if let Some(path) = error.path() {
                println!(" ({})", path.display());
            }
            exit(1);
        });

        let path = entry.path();
        let path = path.to_str().unwrap_or_else(|| {
            println!(
                "ERROR: Cannot convert path to UTF-8: {}",
                path.to_string_lossy(),
            );
            exit(1);
        });

        if !path.ends_with(".kr") {
            continue;
        }

        let file = File::open(path).unwrap_or_else(|error| {
            print!("\nERROR: Failed to open file {} ({})\n\n", path, error,);
            exit(1);
        });

        let stdout = Box::new(stdout());
        let stderr = Box::new(stderr());

        let success = Interpreter::new(stdout, stderr)
            .with_default_builtins()
            .with_default_prelude(&mut ())
            .unwrap_or_else(|error| {
                println!("ERROR: Failed to load prelude: {}", error);
                exit(1);
            })
            .with_default_modules()
            .run(&mut (), path.into(), Box::new(file));

        results.push((success, path.to_owned()));
    }

    for (success, path) in results {
        if success.is_ok() {
            println!(
                "       {}{}OK{}{} {}",
                style::Bold,
                color::Fg(color::LightGreen),
                color::Fg(color::Reset),
                style::Reset,
                path,
            );
        } else {
            println!(
                "    {}{}ERROR{}{} {}",
                style::Bold,
                color::Fg(color::Red),
                color::Fg(color::Reset),
                style::Reset,
                path,
            );
        }
    }

    println!();
}
