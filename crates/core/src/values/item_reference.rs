use crate::{IssuingAgency, NonEmptyString};

/// An item identified under a registered scheme (`BT-157`).
///
/// An identifier is meaningless without naming its scheme (a GTIN, an SKU, and so on),
/// which the validator requires.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemReference {
    /// The item identifier value.
    pub id: Option<NonEmptyString>,
    /// The issuing agency (`schemeID`) the identifier belongs to.
    pub issuer: Option<IssuingAgency>,
}
