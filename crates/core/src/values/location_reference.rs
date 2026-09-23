use crate::{IssuingAgency, NonEmptyString};

/// A location identified by an agency-issued number (`BT-71`).
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
