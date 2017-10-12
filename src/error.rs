use hyper;
use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    General,
    Http(hyper::Error),
    Io(io::Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::General => "Unknown error",
            Error::Http(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::General => None,
            Error::Http(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
        }
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::General => write!(f, "unknown error"),
            Error::Http(ref e) => write!(f, "HTTP error: {}", e),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::Http(err)
    }
}

impl From<hyper::error::UriError> for Error {
    fn from(err: hyper::error::UriError) -> Error {
        Error::Http(hyper::Error::Uri(err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
