use std::io;

use glob::{GlobError, PatternError};
use ncmdump::error::Errors;
use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("Can't resolve the path")]
    Path,
    #[error("Invalid file format")]
    Format,
    #[error("No file can be converted")]
    NoFile,
    #[error("Can't get file's metadata")]
    Metadata,
    #[error("Worker can't less than 0 and more than 8")]
    Worker,
    #[error("Dump err: {0}")]
    Dump(String),
    #[error("Directory or files cannot be put at the same time")]
    DirOrFiles,
}

impl From<PatternError> for Error {
    fn from(_: PatternError) -> Self {
        Self::Path
    }
}

impl From<GlobError> for Error {
    fn from(_: GlobError) -> Self {
        Self::Path
    }
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Self::Path
    }
}

impl From<Errors> for Error {
    fn from(err: Errors) -> Self {
        Error::Dump(err.to_string())
    }
}
