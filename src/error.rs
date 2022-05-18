use thiserror::Error;

/// The error type for ncmdump.
#[derive(Debug, Error)]
pub enum Errors {
    /// The format of file is invalid
    #[error("Invalid file type")]
    InvalidFile,

    /// Can't decode modify of this file
    #[error("Can't decode modify")]
    ModifyDecodeError,

    /// Unknown error
    #[error("Unknown error")]
    Unknown,
}
