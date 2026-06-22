use crate::{IssuingAgency, NonEmptyString};

/// A standard item reference (`BT-157`): an item identified under a registered scheme.
///
/// The issuer is mandatory here: an identifier is meaningless without naming its scheme
/// (a GTIN, an SKU, and so on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemReference {
    /// The item identifier value.
    pub id: NonEmptyString,
    /// The issuing agency (`schemeID`) the identifier belongs to.
    pub issuer: IssuingAgency,
}
