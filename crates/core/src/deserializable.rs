use crate::{DocumentBuilder, Error, Parser};
use crate::{Format, Namespace};

/// Restores an invoice type from the XML of the binding `F`.
pub trait Deserializable<F: Format, N: Namespace + From<F::Namespace>>: Sized {
    /// Reads the whole document from the parser into a builder,
    /// from the root element down, filling the dictionary in the same pass.
    fn deserialize(parser: &mut Parser<F, N>) -> Result<DocumentBuilder<Self>, Error>;
}
