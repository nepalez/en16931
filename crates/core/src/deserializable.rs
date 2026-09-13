use crate::{Document, Error, Format};

/// Restores an invoice type from the XML of the binding `F`.
pub trait Deserializable<F: Format>: Sized {
    /// Reads the XML of the document into its invoice,
    /// filling the dictionary and the abbreviation table in the same pass.
    fn deserialize(document: &mut Document<Self, F>) -> Result<(), Error>;
}
