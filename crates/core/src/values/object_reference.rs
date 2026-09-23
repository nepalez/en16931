use crate::{InvoicedObjectType, NonEmptyString};

/// An identifier of an object the invoice or line refers to,
/// such as a contract, a subscription, or a meter (`BT-18` document, `BT-128` line).
///
/// Without the scheme the identifier is contextual.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectReference {
    /// Object identifier (`BT-18` / `BT-128`).
    pub id: Option<NonEmptyString>,
    /// Scheme (`BT-18-1` / `BT-128-1`).
    pub scheme: Option<InvoicedObjectType>,
}
