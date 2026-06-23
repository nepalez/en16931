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

/// A record-form path: the dialect-free, namespace-resolved address of one node.
///
/// The path is binding-specific:
/// a UBL path and a CII path never compare equal, even for the same business term.
/// It is the dictionary key an SVRL location resolves against.
/// It renders with a leading slash before every step,
/// such as `/Q{INV}Invoice[1]/Q{CAC}InvoiceLine[2]/Q{CBC}ID[1]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// The ordered steps from the document root down to the addressed node.
    pub steps: Vec<Step>,
}

impl Display for Path {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for step in &self.steps {
            write!(formatter, "/{step}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn step(namespace: Namespace, name: &str, index: usize) -> Step {
        Step {
            namespace,
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    // The identifier of the second invoice line (`BT-126`) in a UBL document.
    fn ubl_line_id() -> Path {
        Path {
            steps: vec![
                step(Namespace::Invoice, "Invoice", 1),
                step(Namespace::CommonAggregateComponents, "InvoiceLine", 2),
                step(Namespace::CommonBasicComponents, "ID", 1),
            ],
        }
    }

    // The same business term in a CII document, with its own binding vocabulary.
    fn cii_line_id() -> Path {
        Path {
            steps: vec![
                step(Namespace::CrossIndustryInvoice, "CrossIndustryInvoice", 1),
                step(
                    Namespace::CrossIndustryInvoice,
                    "SupplyChainTradeTransaction",
                    1,
                ),
                step(
                    Namespace::ReusableAggregateBusinessInformationEntity,
                    "IncludedSupplyChainTradeLineItem",
                    2,
                ),
                step(
                    Namespace::ReusableAggregateBusinessInformationEntity,
                    "AssociatedDocumentLineDocument",
                    1,
                ),
                step(
                    Namespace::ReusableAggregateBusinessInformationEntity,
                    "LineID",
                    1,
                ),
            ],
        }
    }

    #[test]
    fn renders_a_ubl_path_in_record_form() {
        assert_eq!(
            ubl_line_id().to_string(),
            "/Q{INV}Invoice[1]/Q{CAC}InvoiceLine[2]/Q{CBC}ID[1]"
        );
    }

    #[test]
    fn renders_a_cii_path_in_record_form() {
        assert_eq!(
            cii_line_id().to_string(),
            "/Q{RSM}CrossIndustryInvoice[1]/Q{RSM}SupplyChainTradeTransaction[1]/Q{RAM}IncludedSupplyChainTradeLineItem[2]/Q{RAM}AssociatedDocumentLineDocument[1]/Q{RAM}LineID[1]"
        );
    }

    #[test]
    fn keeps_ubl_and_cii_paths_distinct() {
        assert_ne!(ubl_line_id(), cii_line_id());
    }

    #[test]
    fn abbreviates_each_namespace() {
        assert_eq!(Namespace::Invoice.to_string(), "INV");
        assert_eq!(Namespace::CommonAggregateComponents.to_string(), "CAC");
        assert_eq!(Namespace::CommonBasicComponents.to_string(), "CBC");
        assert_eq!(Namespace::CrossIndustryInvoice.to_string(), "RSM");
        assert_eq!(
            Namespace::ReusableAggregateBusinessInformationEntity.to_string(),
            "RAM"
        );
    }
}
