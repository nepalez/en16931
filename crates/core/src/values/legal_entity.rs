use crate::{IssuingAgency, NonEmptyString};

/// A legal entity reference: a party as a registered legal entity or person
/// (`BT-30` seller, `BT-47` buyer, `BT-61` payee).
///
/// The identifier comes from an official registrar (a company register).
/// Without the issuer the identifier is contextual.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegalEntity {
    /// The legal registration value.
    pub id: Option<NonEmptyString>,
    /// The issuing agency (`schemeID`), absent for a contextual identifier.
    pub issuer: Option<IssuingAgency>,
}
