use std::{
    fs::File,
    process::exit,
};

use walkdir::WalkDir;

use kari::interpreter::evaluator::Evaluator;


fn main() {
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

        let file = File::open(path)
            .unwrap_or_else(|error| {
                print!(
                    "\nERROR: Failed to open file {} ({})\n\n",
                    path,
                    error,
                );
                exit(1);
            });

        let success = Evaluator::run(path.into(), Box::new(file));
        if success {
            print!("    OK {}\n", path);
        }
    }

    print!("\n");
}
