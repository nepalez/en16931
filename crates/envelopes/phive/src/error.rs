use crate::prelude::*;

/// The failure of the phive envelope reader.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An answer that is not a report of the service.
    Malformed(String),
    /// A finding whose address the report omits.
    MissingLocation(String),
    /// A finding whose level the reader does not know.
    UnknownLevel(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(s) => {
                write!(formatter, "malformed phive report: {s}")
            }
            Self::MissingLocation(s) => {
                write!(
                    formatter,
                    "the phive report finding carries no location: {s}"
                )
            }
            Self::UnknownLevel(s) => {
                write!(formatter, "unknown phive report level: {s}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for CoreError {
    fn from(value: Error) -> Self {
        Self::UnreadableReport(Box::new(value))
    }
}
