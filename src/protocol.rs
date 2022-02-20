//! Communication protocol.
//! 
pub(crate) mod resp;

use std::fmt;

/// Communication error.
#[derive(Debug)]
pub(crate) enum Error {
    /// Data parsing errors.
    ProtcolError,
    /// Connection closed.
    ConnectionClosed,
}

/// Implementation of "Display" for communication errors.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match self {
            ProtcolError => write!(f, "ProtcolError"),
            ConnectionClosed => write!(f, "ConnectionClosed"),
        }
    }
}

/// Implementation of "Error" for communication errors.
impl std::error::Error for Error {}