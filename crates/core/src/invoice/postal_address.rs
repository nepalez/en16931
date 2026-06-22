use crate::{CountryCode, NonEmptyString};

/// A postal address (`BG-5`/`BG-8`/`BG-15`): where a party is located or goods are delivered.
///
/// Only the country is mandatory (`BR-09` and kin), every other line is optional.
/// The model carries the superset of all profiles,
/// so a profile may forbid some of these on serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalAddress {
    /// Address line 1 (`BT-35`).
    pub line1: Option<NonEmptyString>,
    /// Address line 2 (`BT-36`).
    pub line2: Option<NonEmptyString>,
    /// Address line 3 (`BT-162`).
    pub line3: Option<NonEmptyString>,
    /// City (`BT-37`).
    pub city: Option<NonEmptyString>,
    /// Country code (`BT-40`).
    pub country: CountryCode,
    /// Country subdivision (`BT-39`).
    pub country_subdivision: Option<NonEmptyString>,
    /// Post code (`BT-38`).
    pub postal_code: Option<NonEmptyString>,
}
