use hyper;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum KrankerlError {
    General,
    Http(hyper::Error),
    Io(io::Error),
}

impl fmt::Display for KrankerlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KrankerlError::General => write!(f, "unknown error"),
            KrankerlError::Http(ref e) => write!(f, "HTTP error: {}", e),
            KrankerlError::Io(ref e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<hyper::Error> for KrankerlError {
    fn from(err: hyper::Error) -> KrankerlError {
        KrankerlError::Http(err)
    }
}

impl From<hyper::error::UriError> for KrankerlError {
    fn from(err: hyper::error::UriError) -> KrankerlError {
        KrankerlError::Http(hyper::Error::Uri(err))
    }
}

impl From<io::Error> for KrankerlError {
    fn from(err: io::Error) -> KrankerlError {
        KrankerlError::Io(err)
    }
}
