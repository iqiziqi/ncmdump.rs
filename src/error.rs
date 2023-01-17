use thiserror::Error;

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
    #[error("Invalid modify area length")]
    InvalidModifyLength,

    /// The image area is too small
    #[error("Invalid image area length")]
    InvalidImageLength,

    /// Can't decode modify of this file
    #[error("Can't decode modify")]
    ModifyDecodeError,

    /// Unknown error
    #[error("Unknown error")]
    Unknown,
}
