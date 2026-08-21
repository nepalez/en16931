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
    /// An answer of a validator whose entries could not be read.
    UnreadableReport(Box<dyn std::error::Error + Send + Sync>),
    /// An address of a report entry that could not be rewritten into the normalized form.
    UnreadableLocation(Box<dyn std::error::Error + Send + Sync>),
    /// An address of a report entry that binds to no node of the checked document,
    /// kept as the validator wrote it.
    UnboundLocation(String),
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
            Self::UnreadableReport(source) => {
                write!(formatter, "unreadable report: {source}")
            }
            Self::UnreadableLocation(source) => {
                write!(formatter, "unreadable location: {source}")
            }
            Self::UnboundLocation(s) => {
                write!(formatter, "unbound location: {s}")
            }
        }
    }
}

impl std::error::Error for Error {
    /// The failure an extension reported, kept for a consumer that wants its own type.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnreadableReport(source) | Self::UnreadableLocation(source) => {
                Some(source.as_ref())
            }
            _ => None,
        }
    }
}
