//! A quantity with its unit of measure (`BT-129`+`BT-130`, `BT-149`+`BT-150`).
//!
//! The unit code list is UN/ECE Recommendation 20 (units) with the Recommendation 21 extension
//! (package codes), so the [`Unit`] type unites a reused [`UnitOfMeasure`] and a local
//! [`PackageCode`].
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-23`.

use crate::prelude::*;
use crate::{Error, PackageCode};

/// A quantity: a numeric value paired with the unit it is measured in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantity {
    /// The unit the value is measured in.
    pub unit: Unit,
    /// The numeric value of the quantity.
    pub value: Decimal,
}

/// The unit of an invoiced quantity (`BT-130`): either a unit of measure (UN/ECE Rec 20)
/// or a package type code (UN/ECE Rec 21). The union is the loosest `BR-CL-23` constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// A unit of measure (UN/ECE Recommendation 20).
    UnitOfMeasure(UnitOfMeasure),
    /// A package type code (UN/ECE Recommendation 21).
    PackageCode(PackageCode),
}

impl Unit {
    /// Resolves a `BT-130` code, trying a unit of measure first, then a package code.
    pub fn from_code(code: &str) -> Option<Self> {
        UnitOfMeasure::from_code(code)
            .map(Self::UnitOfMeasure)
            .or_else(|| PackageCode::from_code(code).map(Self::PackageCode))
    }

    /// The code of this unit.
    pub fn code(&self) -> &str {
        match self {
            Self::UnitOfMeasure(unit) => unit.code(),
            Self::PackageCode(package) => package.code(),
        }
    }

    /// The human-readable name of this unit.
    pub fn name(&self) -> &str {
        match self {
            Self::UnitOfMeasure(unit) => unit.name(),
            Self::PackageCode(package) => package.name(),
        }
    }
}

impl FromStr for Unit {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_code(value).ok_or_else(|| Error::invalid_value(format!("{value:?}")))
    }
}

impl From<UnitOfMeasure> for Unit {
    fn from(value: UnitOfMeasure) -> Self {
        Self::UnitOfMeasure(value)
    }
}

impl From<PackageCode> for Unit {
    fn from(value: PackageCode) -> Self {
        Self::PackageCode(value)
    }
}

impl TryFrom<&str> for Unit {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.code().fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_unit_of_measure() {
        let unit: Unit = "KGM".parse().expect("KGM is a valid unit");

        assert!(matches!(unit, Unit::UnitOfMeasure(_)));
        assert_eq!(unit.code(), "KGM");
    }

    #[test]
    fn parses_a_package_code() {
        let unit: Unit = "XBA".parse().expect("XBA is a valid unit");

        assert!(matches!(unit, Unit::PackageCode(_)));
        assert_eq!(unit.name(), "Barrel");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("XZZZ".parse::<Unit>().is_err());
    }

    #[test]
    fn wraps_a_unit_of_measure() {
        let measure = UnitOfMeasure::from_code("KGM").expect("KGM is a unit of measure");

        assert_eq!(Unit::from(measure), Unit::UnitOfMeasure(measure));
    }

    #[test]
    fn wraps_a_package_code() {
        let package = PackageCode::from_code("XBA").expect("XBA is a package code");

        assert_eq!(Unit::from(package), Unit::PackageCode(package));
    }
}
