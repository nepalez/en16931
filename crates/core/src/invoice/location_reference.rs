use crate::{IssuingAgency, NonEmptyString};

/// A delivery location reference (`BT-71`):
/// a location identified by an agency-issued number.
///
/// The issuer is optional. Without it the identifier is contextual,
/// agreed between the parties rather than resolvable through a registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationReference {
    /// The location identifier value.
    pub id: NonEmptyString,
    /// The issuing agency (`schemeID`), absent for a contextual identifier.
    pub issuer: Option<IssuingAgency>,
}
