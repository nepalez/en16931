use crate::prelude::*;

/// The single error type of the core.
///
/// A variant names a failure point of the core's own logic, never an extension as a category.
/// The failure that caused it, whether it came from a dependency or from an extension,
/// stays reachable through `std::error::Error::source`.
#[derive(Debug)]
pub enum Error {
    /// A value rejected by a domain type for breaking its constraint.
    InvalidValue {
        /// The wording of the rejection.
        message: String,
        /// The failure that caused it, when a parser reported one.
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    /// A malformed XML document, or one with an unrecognized binding, rejected on parse.
    MalformedXml {
        /// The wording of the rejection.
        message: String,
        /// The failure that caused it, when a reader reported one.
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
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

impl Error {
    /// A domain-type rejection of its own, with no failure behind it.
    pub(crate) fn invalid_value(message: impl Into<String>) -> Self {
        Self::InvalidValue {
            message: message.into(),
            source: None,
        }
    }

    /// A parse rejection of the core's own, with no failure behind it.
    pub(crate) fn malformed_xml(message: impl Into<String>) -> Self {
        Self::MalformedXml {
            message: message.into(),
            source: None,
        }
    }

    /// The same rejection, keeping the failure a dependency reported.
    ///
    /// A variant that carries no source of its own stays as it is.
    pub(crate) fn caused_by(self, cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        match self {
            Self::InvalidValue { message, .. } => Self::InvalidValue {
                message,
                source: Some(Box::new(cause)),
            },
            Self::MalformedXml { message, .. } => Self::MalformedXml {
                message,
                source: Some(Box::new(cause)),
            },
            other => other,
        }
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::invalid_value(value.to_string()).caused_by(value)
    }
}

impl From<XmlError> for Error {
    fn from(value: XmlError) -> Self {
        Self::malformed_xml(value.to_string()).caused_by(value)
    }
}

impl From<AttrError> for Error {
    fn from(value: AttrError) -> Self {
        Self::malformed_xml(value.to_string()).caused_by(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { message, .. } => {
                write!(formatter, "invalid value: {message}")
            }
            Self::MalformedXml { message, .. } => {
                write!(formatter, "malformed XML: {message}")
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
    /// The failure a dependency or an extension reported,
    /// kept for a consumer that wants its own type.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidValue { source, .. } | Self::MalformedXml { source, .. } => {
                source.as_deref().map(|cause| cause as _)
            }
            Self::UnreadableReport(source) | Self::UnreadableLocation(source) => {
                Some(source.as_ref())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Binding, Profile};

    // A failure of an extension, which reaches the core through one of its variants.
    #[derive(Debug)]
    struct ExtensionError;

    impl Display for ExtensionError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            write!(formatter, "the extension gave up")
        }
    }

    impl std::error::Error for ExtensionError {}

    #[test]
    fn keeps_the_failure_of_a_domain_type() {
        let rejected = "an unknown specification"
            .parse::<Profile>()
            .expect_err("a rejected identifier");

        let error = Error::from(rejected);

        assert!(matches!(error, Error::InvalidValue { .. }));
        assert!(
            std::error::Error::source(&error)
                .expect("the source of the rejection")
                .downcast_ref::<ParseError>()
                .is_some()
        );
    }

    #[test]
    fn keeps_the_failure_of_the_xml_reader() {
        let error = Binding::detect("<Invoice").expect_err("a rejected document");

        assert!(matches!(error, Error::MalformedXml { .. }));
        assert!(
            std::error::Error::source(&error)
                .expect("the source of the rejection")
                .downcast_ref::<XmlError>()
                .is_some()
        );
    }

    #[test]
    fn keeps_the_failure_of_an_extension() {
        let error = Error::UnreadableReport(Box::new(ExtensionError));

        assert!(
            std::error::Error::source(&error)
                .expect("the source of the failure")
                .downcast_ref::<ExtensionError>()
                .is_some()
        );
    }

    #[test]
    fn keeps_no_source_of_a_rejection_of_its_own() {
        let error = Error::malformed_xml("no root element");

        assert!(std::error::Error::source(&error).is_none());
    }
}
