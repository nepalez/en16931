use crate::{InvoicedObjectType, NonEmptyString};

/// An invoiced object reference (`BT-18` document, `BT-128` line):
/// an identifier of an object the invoice or line refers to,
/// such as a contract, a subscription, or a meter.
///
/// The scheme is optional; without it the identifier is contextual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectReference {
    /// Object identifier (`BT-18` / `BT-128`).
    pub id: NonEmptyString,
    /// Scheme (`BT-18-1` / `BT-128-1`).
    pub scheme: Option<InvoicedObjectType>,
}
