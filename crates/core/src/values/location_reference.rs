use crate::{IssuingAgency, NonEmptyString};

/// A delivery location reference (`BT-71`):
/// a location identified by an agency-issued number.
///
/// Without the issuer the identifier is contextual,
/// agreed between the parties rather than resolvable through a registry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationReference {
    /// The location identifier value.
    pub id: Option<NonEmptyString>,
    /// The issuing agency (`schemeID`), absent for a contextual identifier.
    pub issuer: Option<IssuingAgency>,
}
