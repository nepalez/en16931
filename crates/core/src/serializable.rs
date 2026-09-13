use crate::{Document, Format};

/// Renders an invoice type into the XML of the binding `F`.
pub trait Serializable<F: Format>: Sized {
    /// Formats a document's invoice as XML,
    /// filling the dictionary and the abbreviation table in the same pass.
    fn serialize(document: &mut Document<Self, F>);
}
