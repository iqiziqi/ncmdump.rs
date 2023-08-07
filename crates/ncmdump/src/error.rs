use std::io;

use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Errors>;

/// The error type for ncmdump.
#[derive(Debug, Error)]
pub enum Errors {
    /// The format of file is invalid
    #[error("Invalid file type")]
    InvalidFileType,

    /// The key area is too small
    #[error("Invalid key area length")]
    InvalidKeyLength,

    /// The music info area is too small
    #[error("Invalid info area length")]
    InvalidInfoLength,

    /// The image area is too small
    #[error("Invalid image area length")]
    InvalidImageLength,

    /// Can't decode information of this file
    #[error("Can't decode information")]
    InfoDecodeError,

    /// Can't decrypt data
    #[error("Can't decrypt")]
    DecryptError,

    /// Unknown error
    #[error("Unknown error")]
    Unknown,

    /// Decode error
    #[error("Decode error")]
    Decode,

    /// IO error
    #[error("IO Error: {0}")]
    IO(String),
}

impl From<io::Error> for Errors {
    fn from(value: io::Error) -> Self {
        Self::IO(value.to_string())
    }
}
