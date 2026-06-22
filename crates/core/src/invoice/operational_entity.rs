//! An entity reference (`BT-29` seller, `BT-46` buyer, `BT-60` payee).

use crate::{IssuingAgency, NonEmptyString};

/// An entity reference:
/// an organization identified operationally by an agency-issued number
/// (`BT-29` seller, `BT-46` buyer, `BT-60` payee).
///
/// The same organization in its registered legal capacity is a `LegalEntity` instead.
///
/// The issuer is optional.
/// Without it the identifier is contextual, agreed between the parties
/// rather than resolvable through a registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalEntity {
    /// The identifier value.
    pub id: NonEmptyString,
    /// The issuing agency (`schemeID`), absent for a contextual identifier.
    pub issuer: Option<IssuingAgency>,
}
