//! A monetary amount: a decimal value with its currency.

use crate::prelude::*;

/// A monetary amount: a decimal value paired with its currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Amount {
    pub value: Decimal,
    pub currency: Currency,
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency.code())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn displays_the_value_with_the_currency() {
        let amount = Amount {
            value: Decimal::new(1050, 2),
            currency: Currency::EUR,
        };

        assert_eq!(amount.to_string(), "10.50 EUR");
    }
}
