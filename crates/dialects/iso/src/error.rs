use crate::prelude::*;

/// The failure of the ISO-skeleton address reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An address the grammar of the dialect does not cover.
    Malformed(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(s) => {
                write!(
                    formatter,
                    "the address lies outside the ISO-skeleton grammar: {s}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for CoreError {
    fn from(value: Error) -> Self {
        Self::UnreadableLocation(Box::new(value))
    }
}
