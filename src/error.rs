use thiserror::Error;

/// The error type for ncmdump.
#[derive(Debug, Error)]
pub enum Errors {
    /// The format of file is invalid
    #[error("Invalid file type")]
    InvalidFile,

    #[error("Invalid file content: {0}")]
    Invalid(String),

    /// Can't decode modify of this file
    #[error("Can't decode modify")]
    ModifyDecodeError,

    /// Unknown error
    #[error("Unknown error")]
    Unknown,
}
