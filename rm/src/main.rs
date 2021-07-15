use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{App, Arg, ArgMatches};
use glob::glob;

mod util;

use util::UnwrapOrExitWith;

struct RmBuilder {
    force: bool,
    prompt: bool,
    recursive: bool,
    quiet: bool,
    verbose: bool,
}

impl RmBuilder {
    fn rm(&self, path_iter: impl IntoIterator<Item = PathBuf>) -> io::Result<()> {
        let mut input_buf = String::new();

        for path in path_iter.into_iter() {
            if self.prompt
                && !ask_affirmation(
                    format!("Delete file \"{}\"?", path.display()),
                    &mut input_buf,
                )?
            {
                continue;
            }

            match remove_fso(&path, self.force, self.recursive) {
                Ok(_) if self.verbose => {
                    println!("Removed fso \"{}\"", path.display());
                }
                Err(e) => {
                    if !self.quiet {
                        print_err!("failed to remove fso \"{}\". Reason: {}", path.display(), e);
                    }
                    continue;
                }
                _ => (),
            }
        }

        Ok(())
    }
}

impl From<&ArgMatches<'static>> for RmBuilder {
    fn from(args: &ArgMatches) -> Self {
        Self {
            force: args.is_present("force"),
            prompt: args.is_present("prompt"),
            recursive: args.is_present("recursive"),
            quiet: args.is_present("quiet"),
            verbose: args.is_present("verbose"),
        }
    }
}

fn get_args() -> ArgMatches<'static> {
    App::new("rm")
        .version("1.0")
        .author("Liam <liam.henrickson@gmail.com>")
        .about("Removes filesystem objects")
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Disables all prompting before removal"),
        )
        .arg(
            Arg::with_name("prompt")
                .short("p")
                .long("prompt")
                .help("Prompts before every removal"),
        )
        .arg(
            Arg::with_name("no-preserve-root")
                .long("no-preserve-root")
                .help("Allows for deleting of '/' and '/*'"),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Removes directories and their contents recursively"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Disables reporting errors"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Verbose mode; reports each file as it's being deleted"),
        )
        // make this take multiple values
        .arg(Arg::with_name("pattern").required(true))
        .get_matches()
}

fn ask_affirmation(question: impl AsRef<str>, mut input_buf: &mut String) -> io::Result<bool> {
    println!("{}", question.as_ref());

    io::stdin().read_line(&mut input_buf)?;
    input_buf.make_ascii_lowercase();

    Ok(matches!(input_buf.as_str(), "y" | "yes"))
}

// dirs are always removed recursively
fn remove_fso(path: &Path, force: bool, remove_if_dir: bool) -> io::Result<()> {
    // TODO: implement `force` flag.
    if remove_if_dir && path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn main() -> io::Result<()> {
    let args = get_args();
    let pattern = args
        .value_of("pattern")
        .unwrap_or_exit_with("invalid characters in pattern");

    if !args.is_present("no-preserve-root") && matches!(pattern, "/" | "/*") {
        exit_with_err!(
            "Can't delete root ('/') or it's members ('/*') without --no-preserve-root option."
        );
    }

    let path_iter = glob(pattern)
        .unwrap_or_exit_with("invalid pattern")
        .into_iter()
        .filter_map(|r| match r {
            Ok(p) => Some(p),
            Err(e) => {
                print_err!(e);
                None
            }
        });

    RmBuilder::from(&args).rm(path_iter)
}
