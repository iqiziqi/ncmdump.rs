use std::error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        fmt.write_str(self.msg())
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn msg(&self) -> &str {
        match self.kind() {
            ErrorKind::InvalidFile => "Invalid file",
            ErrorKind::ModifyDecodeError => "Can't decode modify",
            ErrorKind::Unknown => "Unknown error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    InvalidFile,
    ModifyDecodeError,
    Unknown,
}
