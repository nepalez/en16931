//! The VAT identifier of a party (`BT-31` seller, `BT-48` buyer).

use crate::prelude::*;
use crate::{Error, NonEmptyString};

/// A VAT identifier (`BT-31` seller, `BT-48` buyer).
///
/// Per `BR-CO-9` the value begins with a two-letter country prefix: an ISO 3166-1 alpha-2 code,
/// or `EL` for Greece. The constructor resolves that prefix into the [`CountryCode`] and stores
/// the national number apart from it. The value is trimmed and upper-cased. A Greek identifier
/// is re-emitted with the `EL` prefix. The national part itself is left to the external validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatIdentifier {
    country: CountryCode,
    number: NonEmptyString,
}

impl VatIdentifier {
    /// Builds a VAT identifier from a country and the bare national number (without the prefix).
    /// The national number is what a party stores; the country prefix is the standard's framing.
    pub fn build(country: CountryCode, number: &str) -> Result<Self, Error> {
        Ok(Self {
            country,
            number: number.parse()?,
        })
    }

    /// The country whose VAT scheme issued this identifier.
    pub fn country_code(&self) -> CountryCode {
        self.country
    }

    /// The national VAT number, without the country prefix.
    pub fn number(&self) -> &str {
        self.number.as_ref()
    }

    // The VAT prefix: `EL` for Greece, the ISO 3166-1 alpha-2 code otherwise.
    fn prefix(&self) -> &'static str {
        match self.country.alpha2() {
            "GR" => "EL",
            other => other,
        }
    }
}

impl FromStr for VatIdentifier {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let code = value.trim().to_ascii_uppercase();
        let bytes = code.as_bytes();
        let has_prefix =
            bytes.len() >= 2 && bytes[0].is_ascii_uppercase() && bytes[1].is_ascii_uppercase();
        if !has_prefix {
            return Err(Error::InvalidValue(format!("{value:?}")));
        }

        let alpha2 = if &code[..2] == "EL" { "GR" } else { &code[..2] };
        let country = CountryCode::for_alpha2(alpha2)
            .map_err(|_| Error::InvalidValue(format!("{value:?}")))?;

        Ok(Self {
            country,
            number: code[2..].parse()?,
        })
    }
}

impl TryFrom<&str> for VatIdentifier {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl Display for VatIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix(), self.number)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_valid_identifier() {
        let vat: VatIdentifier = "DE123456789"
            .parse()
            .expect("a prefixed VAT identifier is valid");

        assert_eq!(
            vat.country_code(),
            CountryCode::for_alpha2("DE").expect("DE is a country code")
        );
        assert_eq!(vat.number(), "123456789");
        assert_eq!(vat.to_string(), "DE123456789");
    }

    #[test]
    fn maps_the_greek_el_prefix_to_its_country() {
        let vat: VatIdentifier = "EL123456789"
            .parse()
            .expect("a Greek VAT identifier is valid");

        assert_eq!(
            vat.country_code(),
            CountryCode::for_alpha2("GR").expect("GR is a country code")
        );
        assert_eq!(vat.to_string(), "EL123456789");
    }

    #[test]
    fn accepts_a_non_eu_country_prefix() {
        assert!("NO123456789".parse::<VatIdentifier>().is_ok());
    }

    #[test]
    fn trims_and_upper_cases_the_value() {
        let vat: VatIdentifier = "  de123456789  "
            .parse()
            .expect("a padded VAT identifier is valid");

        assert_eq!(vat.to_string(), "DE123456789");
    }

    #[test]
    fn rejects_an_unknown_country_prefix() {
        assert!("XX123456789".parse::<VatIdentifier>().is_err());
    }

    #[test]
    fn rejects_a_missing_national_part() {
        assert!("DE".parse::<VatIdentifier>().is_err());
    }

    #[test]
    fn keeps_separators_in_the_national_part() {
        // `BR-CO-9` constrains only the prefix; the national format is the validator's job.
        let vat: VatIdentifier = "DE123-456"
            .parse()
            .expect("a VAT identifier with separators is valid");

        assert_eq!(vat.number(), "123-456");
    }

    #[test]
    fn builds_from_a_country_and_bare_number() {
        let germany = CountryCode::for_alpha2("DE").expect("DE is a country code");
        let vat =
            VatIdentifier::build(germany, "123456789").expect("a German VAT identifier builds");

        assert_eq!(vat.country_code(), germany);
        assert_eq!(vat.to_string(), "DE123456789");
    }

    #[test]
    fn build_emits_the_el_prefix_for_greece() {
        let greece = CountryCode::for_alpha2("GR").expect("GR is a country code");
        let vat = VatIdentifier::build(greece, "123456789").expect("a Greek VAT identifier builds");

        assert_eq!(vat.to_string(), "EL123456789");
    }

    #[test]
    fn build_rejects_an_empty_number() {
        let germany = CountryCode::for_alpha2("DE").expect("DE is a country code");

        assert!(VatIdentifier::build(germany, "  ").is_err());
    }
}
