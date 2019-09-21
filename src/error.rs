use std::error;
use std::fmt::{Display, Formatter, Result};

/// The error type for ncmdump.
///
/// It contains kind and msg fields,
/// you can get it by `kind` method and `msg` method,
/// them are useful.
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

    /// Create a new Error from a kind of error.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate ncmdump;
    ///
    /// use ncmdump::error::{Error, ErrorKind};
    ///
    /// let err = Error::new(ErrorKind::Unknown);
    /// ```
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    /// Get the error kind
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate ncmdump;
    /// # use ncmdump::error::{Error, ErrorKind};
    /// #
    /// # let err = Error::new(ErrorKind::Unknown);
    /// #
    /// println!("{:?}", err.kind());
    ///
    /// // Output: Unknown
    /// ```
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Get the error message
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate ncmdump;
    /// # use ncmdump::error::{Error, ErrorKind};
    /// #
    /// # let err = Error::new(ErrorKind::Unknown);
    /// #
    /// println!("{}", err.msg());
    ///
    /// // Output: Unknown error
    /// ```
    pub fn msg(&self) -> &str {
        match self.kind() {
            ErrorKind::InvalidFile => "Invalid file",
            ErrorKind::ModifyDecodeError => "Can't decode modify",
            ErrorKind::Unknown => "Unknown error",
        }
    }
}

/// A list specifying general categories of ncmdump error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    /// The format of file is invalid
    InvalidFile,
    /// Can't decode modify of this file
    ModifyDecodeError,
    /// Unknown error
    Unknown,
}
