// TODO:
//  - improve errors
//  - flag to create parents for destination

use std::borrow::Cow;
use std::fs;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Parser;
use path_clean::PathClean;
use tempfile::NamedTempFile;

const EDITOR: &str = "hx";

/// Rename a file, opening the old name in an editor if one isn't provided.
#[derive(Parser, Debug)]
struct Args {
    src: PathBuf,
    dst: Option<PathBuf>,
    /// In the editor, use the fully qualified, canonical path instead of the relative path.
    #[arg(short = 'c', long)]
    canonical: bool,
    /// Replace the destination file if it already exists.
    #[arg(short = 'd', long)]
    destructive: bool,
    /// Create ancestor directories of destination if they don't already exist.
    #[arg(short = 'a', long)]
    create_ancestors: bool,
}

fn get_dst_in_editor(src: &Path, use_canonical: bool) -> Result<PathBuf> {
    // Apparently the rust interface for windows paths uses utf8 even though windows used wtf16.
    // Even though `std::os::windows::ffi::OsStrExt` suggests otherwise. I spent way too long trying
    // to juggle the encoding back and forth before I got to the point where it said windows paths
    // can't contain NULs.

    // Handle canonicalizing
    let mut src = Cow::Borrowed(src);
    if use_canonical && src.is_relative() {
        src = Cow::Owned(std::env::current_dir()?.join(src).clean());
    }

    // Make the file and write the placeholder
    let mut tmp_file = NamedTempFile::new().context("Failed to make temp file")?;
    tmp_file
        .write_all(src.to_string_lossy().as_bytes())
        .context("Failed to write placeholder to temp file")?;

    // Open the editor
    Command::new(EDITOR)
        .arg(tmp_file.path())
        .status()
        .with_context(|| {
            format!("Failed to launch editor (it's set to '{EDITOR}'; is it in your $PATH?)")
        })?;

    tmp_file.seek(io::SeekFrom::Start(0))?;
    let mut buf = String::new();
    tmp_file
        .read_to_string(&mut buf)
        .context("Failed to read from temp file")?;

    Ok(PathBuf::from(buf))
}

fn run(args: Args) -> Result<()> {
    if !args.src.exists() {
        bail!("\"{}\" doesn't exist", args.src.display());
    }

    let dst = match args.dst {
        Some(dst) => dst,
        None => get_dst_in_editor(&args.src, args.canonical)?,
    };

    if !args.destructive && dst.exists() {
        bail!(
            "\"{}\" already exists. Use the '-d' flag to replace anyway.",
            dst.display()
        );
    }
    if args.create_ancestors {
        fs::create_dir_all(&dst).context("Failed to create ancestors")?;
    }

    if args.src != dst {
        Ok(std::fs::rename(args.src, dst).context("Failed to rename.")?)
    } else {
        Ok(())
    }
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("Error: {e}")
    }
}
