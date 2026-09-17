use crate::{IssuingAgency, NonEmptyString};

/// A standard item reference (`BT-157`): an item identified under a registered scheme.
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
