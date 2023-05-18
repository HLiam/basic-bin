use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{arg, Parser};

use common::ExpandUser;

trait PathExt {
    fn touch(self) -> io::Result<()>;
}
impl PathExt for PathBuf {
    fn touch(self) -> io::Result<()> {
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self)
            .map(|_| ())
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Create ancestor directories of destination if they don't already exist
    #[arg(short = 'a', long)]
    create_ancestors: bool,
    path: PathBuf,
}

fn run(mut args: Args) -> Result<()> {
    use io::ErrorKind::*;

    args.path = args.path.expand_user()?;

    if args.create_ancestors {
        args.path.parent().map(fs::create_dir_all).transpose()?;
    }

    args.path.touch().map_err(|e| {
        anyhow!(match e.kind() {
            NotFound =>
                "Path is invalid. If the ancestor directories don't exist, automatically create \
                them with the '-a' flag",
            InvalidInput => "File name is forbidden",
            _ => return anyhow!(e),
        })
    })
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("Error: {e}");
    }
}
