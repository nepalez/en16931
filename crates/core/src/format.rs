use crate::prelude::*;
use crate::{Context, Namespace, Path};

pub(crate) mod cii;
pub(crate) mod trace;
pub(crate) mod ubl;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use cii::Cii;
pub use ubl::Ubl;

/// An XML binding of the standard, represented by a marker type.
#[allow(private_bounds)]
pub trait Format: Sealed {
    /// The base set of record-form namespaces this binding writes.
    type Namespace: Namespace;
}

pub(crate) trait Sealed {}

/// The dictionary to bind nodes of the XML into the document's ones.
///
/// The key is the node's record-form `Path` over the namespace set `N`.
/// The value is the `Context` for a consumer to highlight.
pub type Dictionary<N> = HashMap<Path<N>, Context>;
