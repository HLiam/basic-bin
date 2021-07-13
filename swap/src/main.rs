use std::fs;
use std::path::Path;

use clap::{App, Arg, ArgMatches};
use tempdir::TempDir;

fn get_args() -> ArgMatches<'static> {
    App::new("swap")
        .version("0.1.0")
        .about("Swap two files")
        .arg(
            Arg::with_name("FILE1")
                .required(true)
                .help("One of the files to swap"),
        )
        .arg(
            Arg::with_name("FILE2")
                .required(true)
                .help("One of the files to swap"),
        )
        .get_matches()
}

fn swap_files(path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> std::io::Result<()> {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    let temp_dir = TempDir::new("swap")?;
    let temp_file = temp_dir.path().join("swap-tmp");

    fs::rename(path2, &temp_file)?;
    fs::rename(path1, path2)?;
    fs::rename(temp_file, path1)
}

fn main() -> std::io::Result<()> {
    let args = get_args();

    swap_files(
        args.value_of_os("FILE1").expect("failed to get value of file 1"),
        args.value_of_os("FILE2").expect("failed to get value of file 2"),
    )
}
