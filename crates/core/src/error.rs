use crate::prelude::*;

/// The single error type of the core.
#[derive(Debug)]
pub enum Error {
    /// A value rejected by a domain type for breaking its constraint.
    InvalidValue(String),
    /// A malformed XML document, or one with an unrecognized binding, rejected on parse.
    MalformedXml(String),
    /// An abbreviation the document binds to a second namespace.
    AmbiguousAbbreviation(String),
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
            Self::MalformedXml(s) => {
                write!(formatter, "malformed XML: {s}")
            }
            Self::AmbiguousAbbreviation(s) => {
                write!(formatter, "ambiguous abbreviation: {s}")
            }
        }
    }
}

impl std::error::Error for Error {}
