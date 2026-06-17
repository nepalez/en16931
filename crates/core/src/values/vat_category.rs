//! The VAT category code of an invoice (subset of `UNCL5305`).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rules `BR-CL-17`/`BR-CL-18`.

use crate::Error;
use crate::prelude::*;

/// The VAT category (subset of `UNCL5305`): how VAT applies to an amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
pub enum VatCategory {
    #[display("S")]
    Standard,
    #[display("Z")]
    ZeroRated,
    #[display("E")]
    Exempt,
    #[display("AE")]
    ReverseCharge,
    #[display("K")]
    IntraCommunitySupply,
    #[display("G")]
    FreeExport,
    #[display("O")]
    OutsideScope,
    #[display("L")]
    CanaryIslands,
    #[display("M")]
    CeutaMelilla,
    #[display("B")]
    TransferredItaly,
}

impl TryFrom<&str> for VatCategory {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_known_code() {
        let category: VatCategory = "AE".parse().expect("AE is a valid VAT category");

        assert_eq!(category, VatCategory::ReverseCharge);
        assert_eq!(category.to_string(), "AE");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("X".parse::<VatCategory>().is_err());
    }

    #[test]
    fn converts_via_try_from() {
        assert_eq!(
            VatCategory::try_from("S").expect("S is a valid VAT category"),
            VatCategory::Standard
        );
        assert!(matches!(
            VatCategory::try_from("X"),
            Err(Error::InvalidValue(_))
        ));
    }
}
