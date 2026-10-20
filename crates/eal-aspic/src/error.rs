//! Distinguish an invalid formal request from an unfinished bounded computation.
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AspicError {
    Invalid(String),
    Incomplete(String),
}

impl fmt::Display for AspicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Incomplete(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AspicError {}

impl From<String> for AspicError {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}

impl From<&str> for AspicError {
    fn from(message: &str) -> Self {
        Self::Invalid(message.to_owned())
    }
}
