use crate::{Binding, DocumentKind, Profile};

/// What a validator needs to pick the rule set for a document.
#[derive(Debug, PartialEq)]
pub struct Target {
    /// The profile the document declares (`BT-24`).
    pub profile: Profile,
    /// The binding the document is serialized in.
    pub binding: Binding,
    /// The kind of the document, an invoice or a credit note (`BT-3`).
    pub kind: DocumentKind,
}
