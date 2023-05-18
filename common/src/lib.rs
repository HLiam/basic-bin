use std::error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HomeDirError {
    NoHomeDir,
    NonUtf8HomeDir,
}

impl error::Error for HomeDirError {}
impl Display for HomeDirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HomeDirError::*;

        write!(
            f,
            "{}",
            match self {
                NoHomeDir => "No home directory set",
                NonUtf8HomeDir => "Home directory path contains invalid unicode characters",
            }
        )
    }
}

pub trait ExpandUser
where
    Self: Sized,
{
    fn expand_user(self) -> Result<Self, HomeDirError>;
}

impl ExpandUser for PathBuf {
    fn expand_user(self) -> Result<Self, HomeDirError> {
        // TODO: change this to use home_dir as a static once std::cell::OnceCell is stabilized.
        Ok(if self.starts_with("~") {
            let mut tmp = dirs::home_dir().ok_or(HomeDirError::NoHomeDir {})?;
            tmp.extend(self.components().skip(1));
            tmp
        } else {
            self
        })
    }
}

impl ExpandUser for String {
    fn expand_user(self) -> Result<Self, HomeDirError> {
        // TODO: we can get rid of this once OnceCell is stabilized.
        fn home_dir_as_string() -> Result<String, HomeDirError> {
            Ok(dirs::home_dir()
                .ok_or(HomeDirError::NoHomeDir)?
                .to_str()
                .ok_or(HomeDirError::NonUtf8HomeDir)?
                .to_owned())
        }

        if self == "~" {
            home_dir_as_string()
        } else if self.starts_with("~/") || self.starts_with("~\\") {
            // TODO: change this to use home_dir as a static once std::cell::OnceCell is stabilized.
            home_dir_as_string().map(|mut i| {
                i.push_str(&self[1..]);
                i
            })
        } else {
            Ok(self)
        }
    }
}
