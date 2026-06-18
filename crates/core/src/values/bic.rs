//! The payment service provider identifier of an invoice (`BT-86`).

use crate::Error;
use crate::prelude::*;

/// A Business Identifier Code (BIC, ISO 9362): the payment service provider identifier (`BT-86`).
///
/// The value is trimmed and upper-cased on construction.
///
/// A BIC is 8 or 11 alphanumeric characters:
/// * four for the institution,
/// * two letters for the country, a valid ISO 3166-1 code,
/// * two for the location,
/// * and an optional three for the branch.
///
/// ISO 9362:2022 allows digits in the institution prefix, so the prefix is unconstrained,
/// while the country positions must form a registered ISO 3166-1 alpha-2 code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bic(String);

impl FromStr for Bic {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let code = value.trim().to_ascii_uppercase();
        let bytes = code.as_bytes();
        let well_formed = matches!(bytes.len(), 8 | 11)
            && bytes.iter().all(u8::is_ascii_alphanumeric)
            && CountryCode::for_alpha2(&code[4..6]).is_ok();

        if well_formed {
            Ok(Self(code))
        } else {
            Err(Error::InvalidValue(format!("{value:?}")))
        }
    }
}

impl TryFrom<&str> for Bic {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl AsRef<str> for Bic {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Bic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_an_eight_character_bic() {
        let bic: Bic = "DEUTDEFF".parse().expect("a valid 8-character BIC parses");

        assert_eq!(bic.as_ref(), "DEUTDEFF");
    }

    #[test]
    fn parses_an_eleven_character_bic_with_branch() {
        let bic: Bic = "DEUTDEFF500"
            .parse()
            .expect("a valid 11-character BIC parses");

        assert_eq!(bic.as_ref(), "DEUTDEFF500");
    }

    #[test]
    fn trims_and_upper_cases_the_value() {
        let bic: Bic = "  deutdeff  ".parse().expect("a padded BIC parses");

        assert_eq!(bic.as_ref(), "DEUTDEFF");
    }

    #[test]
    fn rejects_a_wrong_length() {
        assert!("DEUTDEF".parse::<Bic>().is_err());
    }

    #[test]
    fn rejects_a_digit_in_the_country_position() {
        assert!("DEUT1EFF".parse::<Bic>().is_err());
    }

    #[test]
    fn accepts_a_digit_in_the_institution_prefix() {
        // ISO 9362:2022 allows alphanumeric institution prefixes.
        assert!("1EUTDEFF".parse::<Bic>().is_ok());
    }

    #[test]
    fn rejects_an_unregistered_country() {
        // `QQ` is not a registered ISO 3166-1 code, though it is two letters.
        assert!("DEUTQQFF".parse::<Bic>().is_err());
    }
}
