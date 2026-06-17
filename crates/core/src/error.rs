use crate::prelude::*;

/// The single error type of the core.
#[derive(Debug)]
pub enum Error {
    /// A value rejected by a domain type for breaking its constraint.
    InvalidValue(String),
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::InvalidValue(value.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue(s) => {
                write!(formatter, "invalid value: {s}")
            }
        }
    }
}

impl std::error::Error for Error {}
