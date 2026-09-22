use crate::{DocumentBuilder, Serializer};
use crate::{Format, Namespace};

/// Renders an invoice type into the XML of the binding `F`.
pub trait Serializable<F: Format, N: Namespace + From<F::Namespace>>: Sized {
    /// Writes the whole document of the builder into the serializer, from the root element down,
    /// filling the dictionary and the abbreviation table in the same pass.
    fn serialize(serializer: &mut Serializer<F, N>, builder: &DocumentBuilder<Self>);
}
