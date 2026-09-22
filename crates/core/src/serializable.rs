use crate::Format;
use crate::{DocumentBuilder, Serializer};

/// Renders an invoice type into the XML of the binding `F`.
pub trait Serializable<F: Format>: Sized {
    /// Writes the whole document of the builder into the serializer, from the root element down,
    /// filling the dictionary and the abbreviation table in the same pass.
    fn serialize(serializer: &mut Serializer<F>, builder: &DocumentBuilder<Self>);
}
