use serde::de;
use std::error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Error {
    Custom(String),

    NotSupported,
    InsufficientData,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(format!("{}", msg))
    }
}
