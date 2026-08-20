use crate::prelude::*;

/// The failure of the KoSIT envelope reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An answer that is not a well-formed XML document.
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
                write!(formatter, "malformed KoSIT report: {s}")
            }
            Self::MissingLocation(s) => {
                write!(
                    formatter,
                    "the KoSIT report finding carries no location: {s}"
                )
            }
            Self::UnknownLevel(s) => {
                write!(formatter, "unknown KoSIT report level: {s}")
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
