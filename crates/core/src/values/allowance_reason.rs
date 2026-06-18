//! The document-level allowance reason code
//! (`BT-98`/`BT-140`, subset of `UNCL5189`).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-19`.

use crate::Error;
use crate::prelude::*;

/// The allowance reason (subset of `UNCL5189`): why an allowance is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum AllowanceReason {
    BonusForWorksAheadOfSchedule = 41,
    OtherBonus = 42,
    ManufacturersConsumerDiscount = 60,
    DueToMilitaryStatus = 62,
    DueToWorkAccident = 63,
    SpecialAgreement = 64,
    ProductionErrorDiscount = 65,
    NewOutletDiscount = 66,
    SampleDiscount = 67,
    EndOfRangeDiscount = 68,
    IncotermDiscount = 70,
    PointOfSalesThresholdAllowance = 71,
    MaterialSurchargeDeduction = 88,
    Discount = 95,
    SpecialRebate = 100,
    FixedLongTerm = 102,
    Temporary = 103,
    Standard = 104,
    YearlyTurnover = 105,
}

impl Display for AllowanceReason {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", u16::from(*self))
    }
}

impl FromStr for AllowanceReason {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u16>()
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

impl TryFrom<&str> for AllowanceReason {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl From<AllowanceReason> for u32 {
    fn from(code: AllowanceReason) -> Self {
        u16::from(code).into()
    }
}

impl TryFrom<u32> for AllowanceReason {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u16::try_from(value)
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_known_code() {
        let reason: AllowanceReason = "95".parse().expect("95 is a valid allowance reason");

        assert_eq!(reason, AllowanceReason::Discount);
        assert_eq!(reason.to_string(), "95");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("999".parse::<AllowanceReason>().is_err());
    }

    #[test]
    fn converts_via_try_from() {
        assert_eq!(
            AllowanceReason::try_from("104").expect("104 is a valid allowance reason"),
            AllowanceReason::Standard
        );
        assert!(matches!(
            AllowanceReason::try_from("999"),
            Err(Error::InvalidValue(_))
        ));
    }
}
