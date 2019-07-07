use std::error;
use std::fmt::{Display, Formatter, Result};
use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    InsufficientData,
    Utf8(Utf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8(err)
    }
}
