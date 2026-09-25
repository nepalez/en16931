use crate::prelude::*;
use crate::{Dialect, Envelope, Error, Location};

/// The weight of a report entry.
///
/// Only an error decides the outcome of a check.
/// A document without a single error passes, whatever warnings and remarks it carries.
///
/// The variant renders as its lowercase name, such as `error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Severity {
    /// A breach that invalidates the document.
    #[display("error")]
    Error,
    /// A breach the validator tolerates.
    #[display("warning")]
    Warning,
    /// A remark that reports no breach.
    #[display("information")]
    Information,
}

/// A single finding of a validator report.
///
/// An `Envelope` fills every field but the normalized address.
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
    /// The same address parsed by a `Dialect`, empty until then.
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
    /// The caller pairs an envelope for the service that answered
    /// with a dialect for the processor that wrote the addresses.
    ///
    /// A failure of either one drops the whole report.
    /// It arrives as a core error that keeps its own type behind `std::error::Error::source`.
    pub fn parse<E, D>(report: &str, envelope: &E, dialect: &D) -> Result<Self, Error>
    where
        E: Envelope,
        D: Dialect,
    {
        let mut findings = envelope.unwrap(report).map_err(Into::into)?;
        for finding in &mut findings {
            let location = dialect
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
    use crate::{LocationStep, RawNamespace};

    // Reads one entry per line, and rejects an empty answer.
    struct Service;

    impl Envelope for Service {
        type Error = ServiceError;

        fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error> {
            if report.is_empty() {
                return Err(ServiceError);
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
    struct ServiceError;

    impl Display for ServiceError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            write!(formatter, "the envelope stays closed")
        }
    }

    impl std::error::Error for ServiceError {}

    impl From<ServiceError> for Error {
        fn from(value: ServiceError) -> Self {
            Self::UnreadableReport(Box::new(value))
        }
    }

    // Reads an address of one abbreviated step, and rejects an address without a prefix.
    struct Processor;

    impl Dialect for Processor {
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
        let report = RawReport::parse("/ubl:Invoice\n/cbc:ID", &Service, &Processor)
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
    fn keeps_the_failure_of_an_envelope() {
        let Err(error) = RawReport::parse("", &Service, &Processor) else {
            panic!("the closed envelope should fail the pass");
        };

        assert!(matches!(error, Error::UnreadableReport(_)));
        assert!(
            std::error::Error::source(&error)
                .expect("the source of the failure")
                .downcast_ref::<ServiceError>()
                .is_some()
        );
    }

    #[test]
    fn keeps_the_failure_of_a_dialect() {
        let Err(error) = RawReport::parse("/Invoice", &Service, &Processor) else {
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
