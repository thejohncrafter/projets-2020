
use std::fmt;

use super::line_counter::*;

#[derive(Debug)]
pub struct ReadError<'a> {
    span: Span<'a>,
    message: String,
}

impl<'a> From<(Span<'a>, String)> for ReadError<'a> {
    fn from(desc: (Span<'a>, String)) -> Self {
        ReadError {
            span: desc.0,
            message: desc.1,
        }
    }
}

impl fmt::Display for ReadError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.span, self.message)
    }
}

impl std::error::Error for ReadError<'_> {}

