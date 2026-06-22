use crate::{ItemClassification, NonEmptyString};

/// An item classification (`BT-158`):
/// a code that classifies the item under a registered scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// Classification code (`BT-158`).
    pub id: NonEmptyString,
    /// Scheme (`BT-158-1`).
    pub scheme: ItemClassification,
    /// Scheme list version (`BT-158-2`).
    pub version: Option<NonEmptyString>,
}
