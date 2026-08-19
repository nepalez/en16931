use crate::{Error, Location, Normalizer, Wrapper};

/// The weight of a report entry.
///
/// Only an error decides the outcome of a check.
/// A document without a single error passes, whatever warnings and remarks it carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A breach that invalidates the document.
    Error,
    /// A breach the validator tolerates.
    Warning,
    /// A remark that reports no breach.
    Information,
}

/// A single finding of a validator report.
///
/// A `Wrapper` fills every field but the normalized address.
/// That one comes later, from `RawReport::parse`.
/// The original address survives, so a failure can quote what the validator wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The weight of the finding.
    pub severity: Severity,
    /// The identifier of the rule that fired, when the report names one.
    pub code: Option<String>,
    /// The message the validator wrote for a human reader.
    pub text: String,
    /// The address of the reported node, in the syntax of the processor.
    pub original_location: String,
    /// The same address parsed by a `Normalizer`, empty until then.
    pub normalized_location: Option<Location>,
}

/// A validator report with every address parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawReport {
    /// The findings in the order the validator listed them.
    pub findings: Vec<Entry>,
}

impl RawReport {
    /// Reads the answer of a validator through a pair of extensions.
    ///
    /// The caller pairs a wrapper for the service that answered
    /// with a normalizer for the processor that wrote the addresses.
    ///
    /// A failure of either one drops the whole report.
    /// It arrives as a core error that keeps its own type behind `std::error::Error::source`.
    pub fn parse<W, N>(report: &str, wrapper: &W, normalizer: &N) -> Result<Self, Error>
    where
        W: Wrapper,
        N: Normalizer,
    {
        let mut findings = wrapper.unwrap(report).map_err(Into::into)?;
        for finding in &mut findings {
            let location = normalizer
                .normalize(&finding.original_location)
                .map_err(Into::into)?;
            finding.normalized_location = Some(location);
        }
        Ok(Self { findings })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::{LocationStep, RawNamespace};

    // Reads one entry per line, and rejects an empty answer.
    struct Envelope;

    impl Wrapper for Envelope {
        type Error = EnvelopeError;

        fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error> {
            if report.is_empty() {
                return Err(EnvelopeError);
            }
            Ok(report
                .lines()
                .map(|line| Entry {
                    severity: Severity::Error,
                    code: Some("BR-CO-16".to_owned()),
                    text: "the total does not add up".to_owned(),
                    original_location: line.to_owned(),
                    normalized_location: None,
                })
                .collect())
        }
    }

    #[derive(Debug)]
    struct EnvelopeError;

    impl Display for EnvelopeError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            write!(formatter, "the envelope stays closed")
        }
    }

    impl std::error::Error for EnvelopeError {}

    impl From<EnvelopeError> for Error {
        fn from(value: EnvelopeError) -> Self {
            Self::UnreadableReport(Box::new(value))
        }
    }

    // Reads an address of one abbreviated step, and rejects an address without a prefix.
    struct Dialect;

    impl Normalizer for Dialect {
        type Error = AddressError;

        fn normalize(&self, location: &str) -> Result<Location, Self::Error> {
            let (abbreviation, name) = location
                .trim_start_matches('/')
                .split_once(':')
                .ok_or(AddressError)?;
            Ok(address(abbreviation, name))
        }
    }

    #[derive(Debug)]
    struct AddressError;

    impl Display for AddressError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            write!(formatter, "the address stays unread")
        }
    }

    impl std::error::Error for AddressError {}

    impl From<AddressError> for Error {
        fn from(value: AddressError) -> Self {
            Self::UnreadableLocation(Box::new(value))
        }
    }

    fn address(abbreviation: &str, name: &str) -> Location {
        Location {
            steps: vec![LocationStep {
                namespace: Some(RawNamespace::Abbreviation(abbreviation.to_owned())),
                name: name.to_owned(),
                index: NonZeroUsize::new(1).expect("a positive index"),
            }],
        }
    }

    #[test]
    fn normalizes_the_address_of_every_entry() {
        let report = RawReport::parse("/ubl:Invoice\n/cbc:ID", &Envelope, &Dialect)
            .expect("a report of the pair");

        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.findings[0].severity, Severity::Error);
        assert_eq!(report.findings[0].code.as_deref(), Some("BR-CO-16"));
        assert_eq!(report.findings[0].text, "the total does not add up");
        assert_eq!(report.findings[0].original_location, "/ubl:Invoice");
        assert_eq!(
            report.findings[0].normalized_location,
            Some(address("ubl", "Invoice"))
        );
        assert_eq!(
            report.findings[1].normalized_location,
            Some(address("cbc", "ID"))
        );
    }

    #[test]
    fn keeps_the_failure_of_a_wrapper() {
        let Err(error) = RawReport::parse("", &Envelope, &Dialect) else {
            panic!("the closed envelope should fail the pass");
        };

        assert!(matches!(error, Error::UnreadableReport(_)));
        assert!(
            std::error::Error::source(&error)
                .expect("the source of the failure")
                .downcast_ref::<EnvelopeError>()
                .is_some()
        );
    }

    #[test]
    fn keeps_the_failure_of_a_normalizer() {
        let Err(error) = RawReport::parse("/Invoice", &Envelope, &Dialect) else {
            panic!("the unread address should fail the pass");
        };

        assert!(matches!(error, Error::UnreadableLocation(_)));
        assert!(
            std::error::Error::source(&error)
                .expect("the source of the failure")
                .downcast_ref::<AddressError>()
                .is_some()
        );
    }
}
