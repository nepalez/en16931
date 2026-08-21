//! A percentage rate of an invoice (a VAT rate or an allowance/charge percentage).

use crate::Error;
use crate::prelude::*;

/// A percentage rate: a VAT rate (`BT-96`, `BT-119`, `BT-152`) or an allowance/charge
/// percentage (`BT-94`, `BT-101`, `BT-138`, `BT-143`).
///
/// The value is the rate itself (`19` means 19%, not `0.19`). It is non-negative, since no
/// EN-16931 rate drops below zero. Convert it back to the primitive with `Decimal::from`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Percentage(Decimal);

impl TryFrom<Decimal> for Percentage {
    type Error = Error;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        if value < Decimal::ZERO {
            Err(Error::invalid_value(format!("{value}")))
        } else {
            Ok(Self(value))
        }
    }
}

impl FromStr for Percentage {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let decimal = value
            .trim()
            .parse::<Decimal>()
            .map_err(|_| Error::invalid_value(format!("{value:?}")))?;
        Self::try_from(decimal)
    }
}

impl TryFrom<&str> for Percentage {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl From<Percentage> for Decimal {
    fn from(value: Percentage) -> Self {
        value.0
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_rate() {
        let rate: Percentage = "19.5".parse().expect("a non-negative rate parses");

        assert_eq!(Decimal::from(rate), Decimal::new(195, 1));
    }

    #[test]
    fn accepts_a_zero_rate() {
        assert!("0".parse::<Percentage>().is_ok());
    }

    #[test]
    fn rejects_a_negative_rate() {
        assert!(matches!(
            "-5".parse::<Percentage>(),
            Err(Error::InvalidValue { .. })
        ));
    }

    #[test]
    fn rejects_a_non_numeric_value() {
        assert!("abc".parse::<Percentage>().is_err());
    }
}
