// TODO:
//   - improve errors

use std::fs;
use std::io;

use anyhow::{bail, Context, Result};
use clap::Parser;
use glob::glob;

use common::ExpandUser;

// TODO: add '--dry-run' flag
/// Removes file system objects
#[derive(Parser)]
struct Args {
    // TODO
    /// Disables all prompting before removal
    #[arg(short = 'f', long)]
    force: bool,
    /// Prompts before every removal
    #[arg(short = 'p', long)]
    prompt: bool,
    // TODO
    /// Allows for deleting of '/' and '/*'
    #[arg(long)]
    no_preserve_root: bool,
    /// Removes directories and their contents recursively
    #[arg(short = 'r', long)]
    recursive: bool,
    /// Disables reporting errors
    #[arg(short = 'q', long)]
    quiet: bool,
    /// Verbose mode; reports each file as it's being deleted
    #[arg(short = 'v', long)]
    verbose: bool,
    pattern: String,
}

fn ask_yn(question: impl AsRef<str>, mut buf: &mut String) -> io::Result<bool> {
    println!("{} [y/N]?", question.as_ref());

    loop {
        io::stdin().read_line(&mut buf)?;
        buf.make_ascii_lowercase();

        match buf as &str {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => println!("Invalid input [y/N]?"),
        }
    }
}

fn run(mut args: Args) -> Result<()> {
    let mut input_buf = String::new();

    // TODO: check for protected paths with each canonical path in the glob, too.
    if !args.no_preserve_root && matches!(&args.pattern as &str, "/" | "/*") {
        bail!("Can't delete root ('/') or it's members ('/*') without --no-preserve-root option.");
    }

    // let pattern = if args.pattern.starts_with("~/") matches!(&args.pattern as &str, "~/" | "~\\") {
    args.pattern = args.pattern.expand_user()?.to_owned();
    // } else {
    // args.pattern
    // };

    for p in glob(&args.pattern).context("Invalid pattern")? {
        let p = p.context("Failed to get glob matches")?;
        // TODO: don't use canonicalize here--it follows symlinks.
        let p = match p.canonicalize() {
            Ok(p) => p,
            e if !p.exists() => e?,
            e => e.with_context(|| format!("Failed to canonicalize path \'{}\'", p.display()))?,
        };

        if args.prompt && !ask_yn(format!("Delete file \"{}\"?", p.display()), &mut input_buf)? {
            continue;
        }

        if p.is_file() {
            fs::remove_file(p)
        } else if args.recursive {
            fs::remove_dir_all(p)
        } else {
            fs::remove_dir(p)
        }
        .context("Failed to remove file or dir")?;
    }

    Ok(())
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("Error: {e}")
    }
}
