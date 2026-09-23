use crate::{Binding, InvoiceKind, Profile};

/// What a validator needs to pick the rule set for a document.
#[derive(Debug, PartialEq)]
pub struct Target {
    /// The profile the document declares (`BT-24`).
    pub profile: Profile,
    /// The binding the document is serialized in.
    pub binding: Binding,
    /// The kind of the invoice, a claim for a payment or a credit note.
    pub kind: InvoiceKind,
}
