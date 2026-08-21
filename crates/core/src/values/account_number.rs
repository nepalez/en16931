//! The payment account identifier of an invoice (`BT-84`).

use crate::prelude::*;
use crate::{Error, NonEmptyString};

/// The payment account identifier (`BT-84`): a validated IBAN or another account number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountNumber {
    Iban(Iban),
    Other(NonEmptyString),
}

impl FromStr for AccountNumber {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(iban) = value.trim().parse::<Iban>() {
            return Ok(Self::Iban(iban));
        }
        value.parse::<NonEmptyString>().map(Self::Other)
    }
}

impl TryFrom<&str> for AccountNumber {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl From<Iban> for AccountNumber {
    fn from(iban: Iban) -> Self {
        Self::Iban(iban)
    }
}

impl AsRef<str> for AccountNumber {
    fn as_ref(&self) -> &str {
        match self {
            Self::Iban(iban) => iban.electronic_str(),
            Self::Other(other) => other.as_ref(),
        }
    }
}

impl Display for AccountNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_valid_iban() {
        let identifier: AccountNumber = "DE44500105175407324931"
            .parse()
            .expect("a valid IBAN account number parses");
        let iban: Iban = "DE44500105175407324931"
            .parse()
            .expect("a valid IBAN parses");

        assert_eq!(identifier, AccountNumber::from(iban));
    }

    #[test]
    fn keeps_a_non_iban_value() {
        let identifier: AccountNumber = "ACME-PAYREF-001"
            .parse()
            .expect("a non-IBAN account number parses");

        assert!(matches!(identifier, AccountNumber::Other(_)));
        assert_eq!(identifier.as_ref(), "ACME-PAYREF-001");
    }

    #[test]
    fn rejects_a_blank_value() {
        assert!(matches!(
            "   ".parse::<AccountNumber>(),
            Err(Error::InvalidValue { .. })
        ));
    }

    #[test]
    fn converts_via_try_from() {
        let identifier = AccountNumber::try_from("DE44500105175407324931")
            .expect("a valid IBAN account number parses");

        assert!(matches!(identifier, AccountNumber::Iban(_)));
    }
}
