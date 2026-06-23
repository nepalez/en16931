use crate::prelude::*;

/// An XML namespace of a record-form step,
/// rendered as the short abbreviation of its full namespace URI.
///
/// The variants cover the two bindings and never mix.
/// The abbreviation stands in for the full namespace in a rendered path,
/// so a UBL path and a CII path never compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Namespace {
    /// The UBL namespace of the root `Invoice` document
    /// (`urn:oasis:names:specification:ubl:schema:xsd:Invoice-2`).
    #[display("INV")]
    Invoice,
    /// The UBL Common Aggregate Components namespace,
    /// holding the nested business groups.
    #[display("CAC")]
    CommonAggregateComponents,
    /// The UBL Common Basic Components namespace, holding the leaf fields.
    #[display("CBC")]
    CommonBasicComponents,
    /// The CII namespace of the root `CrossIndustryInvoice` document
    /// and its top-level structural elements.
    #[display("RSM")]
    CrossIndustryInvoice,
    /// The CII Reusable Aggregate Business Information Entity namespace,
    /// holding the business groups and fields.
    #[display("RAM")]
    ReusableAggregateBusinessInformationEntity,
}

/// One step of a record-form path:
/// a namespaced element with its positional index among the same-named siblings.
///
/// It renders as `Q{U}N[i]`, where
/// `U` is the namespace abbreviation,
/// `N` is the local name,
/// `i` is the 1-based index.
///
/// The address is purely positional:
/// a singleton element and the first node of a repeatable group are both index `1`.
/// A normalizer supplies `1` for a location that omits the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The element namespace.
    pub namespace: Namespace,
    /// The element local name, copied verbatim from the location.
    pub name: String,
    /// The 1-based position among the same-named siblings.
    pub index: NonZeroUsize,
}

impl Display for Step {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Q{{{}}}{}[{}]",
            self.namespace, self.name, self.index
        )
    }
}
