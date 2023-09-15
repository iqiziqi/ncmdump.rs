use thiserror::Error;

#[derive(Clone, Debug, Error)]
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
}
