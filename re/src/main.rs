// TODO:
//  - improve errors
//  - tell user that ancestor doesn't exist & suggest nearest name (obv if one doesn't exist)
//      this should probably be mostly done by an separate crate.

use std::fs;
use std::io::{self, Read, Seek, Write};
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Parser;
use path_clean::PathClean;
use tempfile::NamedTempFile;

const EDITOR: &str = "hx";

/// Rename a file, opening the old name in an editor if one isn't provided
#[derive(Parser, Debug)]
struct Args {
    src: PathBuf,
    dst: Option<PathBuf>,
    /// In the editor, use the fully qualified, canonical path instead of the relative path
    #[arg(short = 'c', long)]
    canonical: bool,
    /// Open to only the file name in the editor
    #[arg(short = 'n', long, conflicts_with = "canonical")]
    name_only: bool,
    /// Replace the destination file if it already exists
    #[arg(short = 'd', long)]
    destructive: bool,
    /// Create ancestor directories of destination if they don't already exist
    #[arg(short = 'a', long)]
    create_ancestors: bool,
}

fn get_dst_in_editor(placeholder: &str) -> Result<PathBuf> {
    // Apparently the rust interface for windows paths uses utf8 even though windows used wtf16.
    // Even though `std::os::windows::ffi::OsStrExt` suggests otherwise. I spent way too long trying
    // to juggle the encoding back and forth before I got to the point where it said windows paths
    // can't contain NULs.

    // Make the file and write the placeholder
    let mut tmp_file = NamedTempFile::new().context("Failed to make temp file")?;
    tmp_file
        .write_all(placeholder.as_bytes())
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
    let mut src = args.src;

    if !src.exists() {
        bail!("\"{}\" doesn't exist", src.display());
    }

    if args.canonical && src.is_relative() {
        src = std::env::current_dir()?.join(src).clean();
    }

    let mut dst = match args.dst {
        Some(dst) => dst,
        None => {
            let placeholder = if args.name_only {
                src.file_name()
                    .context("`name-only` flag used with source path that ends in `..`")?
            } else {
                src.as_os_str()
            }
            .to_string_lossy();

            get_dst_in_editor(&placeholder)?
        }
    };
    if dst.as_os_str().is_empty() {
        bail!("Destination can't be an empty string");
    }
    if args.name_only {
        dst = src
            .parent()
            .context("'name-only' flag used with invalid path")?
            .join(dst);
    }
    if !args.destructive && dst.exists() {
        bail!(
            "\"{}\" already exists. Use the '-d' flag to replace anyway",
            dst.display()
        );
    }
    if args.create_ancestors {
        if let Some(p) = dst.parent() {
            fs::create_dir_all(&p).context("Failed to create ancestors")?
        }
    }

    if src != dst {
        Ok(std::fs::rename(src, dst).context("Failed to rename")?)
    } else {
        Ok(())
    }
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("Error: {e}")
    }
}
