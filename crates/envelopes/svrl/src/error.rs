use crate::prelude::*;

/// The failure of the plain SVRL reader.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// A report that is not an SVRL document.
    Malformed(String),
    /// A finding whose address the report omits.
    MissingLocation(String),
    /// A finding whose flag the reader does not know.
    UnknownFlag(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(s) => {
                write!(formatter, "malformed SVRL report: {s}")
            }
            Self::MissingLocation(s) => {
                write!(
                    formatter,
                    "the SVRL report finding carries no location: {s}"
                )
            }
            Self::UnknownFlag(s) => {
                write!(formatter, "unknown SVRL report flag: {s}")
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
