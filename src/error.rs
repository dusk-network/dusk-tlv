use std::error;
use std::fmt;
use std::io;

use serde::de::Error as SerdeDeError;
use serde::ser::Error as SerdeSerError;

macro_rules! from_error {
    ($t:ty, $id:ident) => {
        impl From<$t> for Error {
            fn from(e: $t) -> Self {
                Error::$id(e)
            }
        }
    };
}

/// Standard error for the interface
#[derive(Debug)]
pub enum Error {
    /// I/O [`io::Error`]
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
        }
    }
}

impl SerdeDeError for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("Serde error: {}", msg),
        ))
    }
}

impl SerdeSerError for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("Serde error: {}", msg),
        ))
    }
}

impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        match self {
            Error::Io(e) => e,
        }
    }
}

from_error!(io::Error, Io);
