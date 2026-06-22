use crate::{AllowanceReason, ChargeReason, Decimal, NonEmptyString, Percentage, VatTreatment};

/// A document-level adjustment (`BG-20` allowance, `BG-21` charge):
/// a deduction or addition applied to the whole invoice.
/// The `reason` direction tells an allowance from a charge.
///
/// All its amounts are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adjustment {
    /// Amount (`BT-92`/`BT-99`+`BT-93`+`BT-94`/`BT-100`+`BT-101`).
    pub amount: Amount,
    /// VAT treatment (`BT-95`+`BT-96`/`BT-102`+`BT-103`).
    pub vat: VatTreatment,
    /// Reason and direction (`BT-97`+`BT-98`/`BT-104`+`BT-105`).
    pub reason: Reason,
}

/// A line-level adjustment (`BG-27` allowance, `BG-28` charge):
/// a deduction or addition applied to one invoice line.
/// The line carries its own VAT, so no VAT treatment appears here.
///
/// All its amounts are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineAdjustment {
    /// Amount (`BT-136`/`BT-141`+`BT-137`+`BT-138`/`BT-142`+`BT-143`).
    pub amount: Amount,
    /// Reason and direction (`BT-139`+`BT-140`/`BT-144`+`BT-145`).
    pub reason: Reason,
}

impl LineAdjustment {
    /// The amount signed by direction: positive for a charge, negative for an allowance.
    pub fn signed_amount(&self) -> Decimal {
        match self.reason {
            Reason::Charge { .. } => self.amount.value(),
            Reason::Allowance { .. } => -self.amount.value(),
        }
    }
}

/// The amount of an adjustment: a fixed sum, or a percentage of a base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amount {
    /// A fixed amount (`BT-92`/`BT-99`/`BT-136`/`BT-141`).
    Absolute(Decimal),
    /// A percentage of a base
    /// (`BT-94`/`BT-101`/`BT-138`/`BT-143` of `BT-93`/`BT-100`/`BT-137`/ `BT-142`).
    Relative {
        /// The percentage rate applied to the base.
        rate: Percentage,
        /// The base the rate is applied to.
        base: Decimal,
    },
}

impl Amount {
    /// The adjustment amount (`BT-92`/`BT-99`/`BT-136`/`BT-141`):
    /// the fixed sum, or the base times the rate, rounded.
    pub fn value(&self) -> Decimal {
        match self {
            Self::Absolute(value) => *value,
            Self::Relative { rate, base } => {
                crate::invoice::rounded(base * Decimal::from(*rate) / Decimal::from(100))
            }
        }
    }
}

/// The reason for an adjustment, keyed by its direction: an allowance or a charge.
/// The direction selects the reason code list
/// (`UNCL5189` for an allowance, `UNCL7161` for a charge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// An allowance (a deduction).
    Allowance {
        /// Reason code (`BT-98`/`BT-140`).
        code: Option<AllowanceReason>,
        /// Reason text (`BT-97`/`BT-139`).
        text: Option<NonEmptyString>,
    },
    /// A charge (an addition).
    Charge {
        /// Reason code (`BT-105`/`BT-145`).
        code: Option<ChargeReason>,
        /// Reason text (`BT-104`/`BT-144`).
        text: Option<NonEmptyString>,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn values_a_relative_amount_as_the_base_times_the_rate() {
        let amount = Amount::Relative {
            rate: Percentage::try_from(Decimal::new(15, 1)).expect("a valid rate"),
            base: Decimal::new(20000, 2),
        };

        assert_eq!(amount.value(), Decimal::new(300, 2));
    }

    #[test]
    fn rounds_a_relative_amount_half_away_from_zero() {
        let amount = Amount::Relative {
            rate: Percentage::try_from(Decimal::new(75, 1)).expect("a valid rate"),
            base: Decimal::new(1003, 2),
        };

        // 10.03 * 7.5% = 0.75225 -> 0.75
        assert_eq!(amount.value(), Decimal::new(75, 2));
    }
}
