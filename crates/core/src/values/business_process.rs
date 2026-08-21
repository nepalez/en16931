//! The business process type of an invoice (`BT-23`).

use crate::Error;
use crate::prelude::*;

/// The business process type (`BT-23`): the process context a document targets.
///
/// The value is an identifier, usually a URN like `urn:fdc:peppol.eu:2017:poacc:billing:01:1.0`.
/// The allowed set is open and profile-specific.
/// It leaves the per-profile code list to the external validator.
/// The named constants cover the common process identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessProcess(Cow<'static, str>);

impl BusinessProcess {
    /// Peppol BIS Billing 3.0 process.
    pub const PEPPOL_BILLING: Self =
        Self(Cow::Borrowed("urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"));
    /// Peppol BIS Self-Billing 3.0 process.
    pub const PEPPOL_SELF_BILLING: Self = Self(Cow::Borrowed(
        "urn:fdc:peppol.eu:2017:poacc:selfbilling:01:1.0",
    ));
    /// Peppol PINT billing process.
    pub const PINT_BILLING: Self = Self(Cow::Borrowed("urn:peppol:bis:billing"));
    /// Peppol PINT self-billing process.
    pub const PINT_SELF_BILLING: Self = Self(Cow::Borrowed("urn:peppol:bis:selfbilling"));
}

impl AsRef<str> for BusinessProcess {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for BusinessProcess {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(Error::invalid_value(format!("{value:?}")));
        }
        Ok(Self(Cow::Owned(trimmed.to_owned())))
    }
}

impl TryFrom<&str> for BusinessProcess {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl Display for BusinessProcess {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_process_identifier() {
        let process: BusinessProcess = "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
            .parse()
            .expect("a process identifier parses");

        assert_eq!(
            process.as_ref(),
            "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
        );
    }

    #[test]
    fn equates_a_named_constant_with_its_parsed_value() {
        let parsed: BusinessProcess = "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
            .parse()
            .expect("a process identifier parses");

        assert_eq!(parsed, BusinessProcess::PEPPOL_BILLING);
        assert_eq!(
            BusinessProcess::PEPPOL_BILLING.to_string(),
            "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
        );
    }

    #[test]
    fn rejects_a_blank_value() {
        assert!(matches!(
            "   ".parse::<BusinessProcess>(),
            Err(Error::InvalidValue { .. })
        ));
    }
}
