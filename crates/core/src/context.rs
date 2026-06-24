use crate::prelude::*;

/// One step of a context path: a model field,
/// with the index of the repeatable-group instance it enters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub field: &'static str,
    /// The 1-based index of the entered repeatable-group instance,
    /// or `None` for a single field or group.
    pub index: Option<NonZeroUsize>,
}

/// A pointer to one model node for a consumer to highlight.
///
/// The target is a leaf field, a repeatable-group instance, or the document root.
/// Each segment carries the concrete index of the repeatable group it crosses.
/// An empty path is the document root: either a document-wide rule,
/// or a node with no assigned model term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    /// The ordered segments from the document root down to the target;
    /// empty for the root.
    pub segments: Vec<Segment>,
}
