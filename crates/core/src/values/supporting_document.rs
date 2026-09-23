//! An additional supporting document (`BG-24`).

use crate::{BinaryObject, NonEmptyString, Url};

/// A referenced, linked, or embedded document (`BG-24`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SupportingDocument {
    /// Reference (`BT-122`).
    pub reference: Option<NonEmptyString>,
    /// Description (`BT-123`).
    pub description: Option<NonEmptyString>,
    /// External location (`BT-124`).
    pub external_location: Option<Url>,
    /// Attached document (`BT-125`).
    pub attachment: Option<BinaryObject>,
}
