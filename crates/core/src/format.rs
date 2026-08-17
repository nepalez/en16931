use crate::prelude::*;
use crate::{Context, Path};

pub(crate) mod cii;
pub(crate) mod trace;
pub(crate) mod ubl;

#[cfg(test)]
pub(crate) mod test_helpers;

/// The dictionary to bind nodes of the XML into the document's ones.
///
/// The key is the node's record-form `Path`.
/// The value is the `Context` for a consumer to highlight.
pub type Dictionary = HashMap<Path, Context>;
