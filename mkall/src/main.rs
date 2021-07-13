use std::env;
use std::fs;
use std::process;

fn print_usage() -> ! {
    println!("Usage: mkall <path_or_parts> ...");
    process::exit(2);
}

fn main() -> std::io::Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
    }

    for arg in &args {
        if matches!(arg.as_ref(), "-h" | "--help") {
            print_usage();
        }
    }

    fs::create_dir_all(args.join("/"))
}
