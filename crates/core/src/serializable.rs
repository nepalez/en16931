use crate::{DocumentBuilder, Serializer};
use crate::{Format, Namespace};

/// Renders an invoice type into the XML of the binding `F`.
pub trait Serializable<F: Format, N: Namespace + From<F::Namespace>>: Sized {
    /// Writes the whole document of the builder into the serializer, from the root element down,
    /// filling the dictionary and the abbreviation table in the same pass.
    ///
    /// The builder is taken exclusively because the model hands out its fields
    /// through unique references only. The walk leaves the content unchanged.
    fn serialize(serializer: &mut Serializer<F, N>, builder: &mut DocumentBuilder<Self>);
}
