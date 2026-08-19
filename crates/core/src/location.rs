use crate::prelude::*;

/// A normalized address of a report entry: one location stripped of its dialect.
///
/// It is what a `Normalizer` yields and what the match against a document consumes.
/// Nothing here is resolved yet. A namespace stays in the form its dialect wrote,
/// and a trailing step that addresses no element of the model, such as an attribute,
/// stays in place. Resolving the namespaces and dropping the unmatched tail
/// both belong to the match itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    /// The ordered steps from the document root down to the addressed node.
    pub steps: Vec<LocationStep>,
}

/// One step of a normalized address:
/// an element with its position among the same-named siblings.
///
/// The name and the namespace stay as the address wrote them.
/// The position is purely positional: a singleton element
/// and the first node of a repeatable group are both index `1`.
/// A normalizer supplies `1` for a step whose dialect omits the index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationStep {
    /// The namespace the step belongs to, or `None` for a step that belongs to none.
    ///
    /// An element without a prefix still belongs to the default namespace of the document,
    /// which its table binds to the empty abbreviation.
    ///
    /// `None` therefore stands for something else:
    /// a step outside every namespace, such as an attribute.
    /// No table resolves it, and no node of the document answers it,
    /// so the match stops at the previous step.
    pub namespace: Option<RawNamespace>,
    /// The local name, copied verbatim from the address.
    pub name: String,
    /// The 1-based position among the same-named siblings.
    pub index: NonZeroUsize,
}

/// The namespace of a location step, in the form its dialect wrote it.
///
/// The two forms take different work to resolve.
/// A URI already names the namespace (every namespace is bound to a particular URI).
/// An abbreviation names nothing on its own, and only the table of the checked document binds it.
/// That table covers the abbreviations of the rule sets as well,
/// because a processor writes either its own or the ones the document declares.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RawNamespace {
    /// The full namespace URI.
    Uri(String),
    /// The abbreviation of a namespace, which the document's table resolves.
    Abbreviation(String),
}
