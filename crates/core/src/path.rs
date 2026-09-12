use crate::Namespace;
use crate::prelude::*;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Step<N: Namespace> {
    /// The element namespace.
    pub namespace: N,
    /// The element local name, copied verbatim from the location.
    pub name: String,
    /// The 1-based position among the same-named siblings.
    pub index: NonZeroUsize,
}

impl<N: Namespace> Display for Step<N> {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path<N: Namespace> {
    /// The ordered steps from the document root down to the addressed node.
    pub steps: Vec<Step<N>>,
}

impl<N: Namespace> Display for Path<N> {
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

    use crate::{cii, ubl};

    fn step<N: Namespace>(namespace: N, name: &str, index: usize) -> Step<N> {
        Step {
            namespace,
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    // The identifier of the second invoice line (`BT-126`) in a UBL document.
    fn ubl_line_id() -> Path<ubl::Namespace> {
        Path {
            steps: vec![
                step(ubl::Namespace::Inv, "Invoice", 1),
                step(ubl::Namespace::Cac, "InvoiceLine", 2),
                step(ubl::Namespace::Cbc, "ID", 1),
            ],
        }
    }

    // The same business term in a CII document, with its own binding vocabulary.
    fn cii_line_id() -> Path<cii::Namespace> {
        Path {
            steps: vec![
                step(cii::Namespace::Rsm, "CrossIndustryInvoice", 1),
                step(cii::Namespace::Rsm, "SupplyChainTradeTransaction", 1),
                step(cii::Namespace::Ram, "IncludedSupplyChainTradeLineItem", 2),
                step(cii::Namespace::Ram, "AssociatedDocumentLineDocument", 1),
                step(cii::Namespace::Ram, "LineID", 1),
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
}
