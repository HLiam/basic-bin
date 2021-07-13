use std::env;
use std::fs;
use std::io;
use std::process::exit;

fn touch_file(path: &str) -> io::Result<fs::File> {
    fs::OpenOptions::new().append(true).create(true).open(path)
}

fn main() {
    match env::args().nth(1).as_deref() {
        None | Some("-h" | "--help") => {
            println!("Usage: touch [file]");
        }
        Some(path) => {
            touch_file(path).unwrap_or_else(|err| match err.kind() {
                io::ErrorKind::NotFound => {
                    println!("Error: path contains invalid characters");
                    exit(2);
                }
                io::ErrorKind::InvalidInput => {
                    println!("Error: file name is forbidden");
                    exit(2);
                }
                _ => {
                    panic!("{}", err);
                }
            });
        }
    }
}
