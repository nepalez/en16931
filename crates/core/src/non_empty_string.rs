//! A non-empty string, the base for the identifier field types.

use crate::Error;
use crate::prelude::*;

/// A string carrying a non-empty value, with the surrounding whitespace trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyString(String);

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for NonEmptyString {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(Error::InvalidValue(format!("{value:?}")));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl TryFrom<&str> for NonEmptyString {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl Display for NonEmptyString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keeps_a_value_with_content() {
        let string: NonEmptyString = "INV-001".parse().expect("a value with content is kept");

        let value: &str = string.as_ref();
        assert_eq!(value, "INV-001");
    }

    #[test]
    fn trims_the_surrounding_whitespace() {
        let string: NonEmptyString = "  INV-001  "
            .parse()
            .expect("a padded value with content is kept");

        let value: &str = string.as_ref();
        assert_eq!(value, "INV-001");
    }

    #[test]
    fn rejects_a_blank_value() {
        assert!(matches!(
            "   ".parse::<NonEmptyString>(),
            Err(Error::InvalidValue(_))
        ));
    }

    #[test]
    fn converts_via_try_from() {
        let string = NonEmptyString::try_from("INV-001").expect("a value with content is kept");

        assert_eq!(string.as_ref(), "INV-001");
    }
}
