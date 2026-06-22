use crate::{BinaryObject, NonEmptyString, Url};

/// An additional supporting document (`BG-24`):
/// a referenced, linked, or embedded document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportingDocument {
    /// Reference (`BT-122`).
    pub reference: NonEmptyString,
    /// Description (`BT-123`).
    pub description: Option<NonEmptyString>,
    /// External location (`BT-124`).
    pub external_location: Option<Url>,
    /// Attached document (`BT-125`).
    pub attachment: Option<BinaryObject>,
}
