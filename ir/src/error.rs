
use std::fmt;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error {msg: error}
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Error {msg: format!("{}", error)}
    }
}

