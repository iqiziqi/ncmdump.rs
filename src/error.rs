#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub fn kind(self) -> ErrorKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    FileNotFound,
    InvalidFile,
    ReadOrWrite,
    PermissionDenied,
    Decode,
    Decrypt,
    Unknown,
}
