use crate::Error;
use crate::prelude::*;

/// An XML namespace of a record-form step,
/// rendered as the short abbreviation of its full namespace URI.
///
/// The variants cover the two bindings and never mix.
/// The abbreviation stands in for the full namespace in a rendered path,
/// so a UBL path and a CII path never compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
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

impl Namespace {
    const INV: (&str, &str) = (
        "ubl",
        "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2",
    );
    const CAC: (&str, &str) = (
        "cac",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2",
    );
    const CBC: (&str, &str) = (
        "cbc",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2",
    );
    const CII: (&str, &str) = (
        "rsm",
        "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100",
    );
    const RAM: (&str, &str) = (
        "ram",
        "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100",
    );

    /// The full namespace URI the abbreviation stands for.
    ///
    /// A `Binding` writes it as the element namespace,
    /// and `Binding::detect` matches a document's root element against the two root URIs.
    pub fn uri(self) -> &'static str {
        match self {
            Self::Invoice => Self::INV.1,
            Self::CommonAggregateComponents => Self::CAC.1,
            Self::CommonBasicComponents => Self::CBC.1,
            Self::CrossIndustryInvoice => Self::CII.1,
            Self::ReusableAggregateBusinessInformationEntity => Self::RAM.1,
        }
    }

    /// The namespace whose URI is `uri`, if it is one a binding carries.
    ///
    /// It is the inverse of `uri`, used by a `Binding` parser
    /// to resolve an element's namespace back into its record-form abbreviation.
    pub fn from_uri(uri: &str) -> Option<Self> {
        match uri {
            _ if uri == Self::INV.1 => Some(Self::Invoice),
            _ if uri == Self::CAC.1 => Some(Self::CommonAggregateComponents),
            _ if uri == Self::CBC.1 => Some(Self::CommonBasicComponents),
            _ if uri == Self::CII.1 => Some(Self::CrossIndustryInvoice),
            _ if uri == Self::RAM.1 => Some(Self::ReusableAggregateBusinessInformationEntity),
            _ => None,
        }
    }
}

/// Resolves an abbreviated namespace of a report location.
///
/// A validator may abbreviate the namespace of every step of a location.
/// The abbreviation comes either from the rule set or from the checked document,
/// this is decided by a validator outside the library's control.
///
/// That's why we should know how to map either choice to a proper namepsaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abbreviations(HashMap<String, Namespace>);

impl Abbreviations {
    /// Adds an abbreviation the document binds.
    ///
    /// Rebinding one to the same namespace changes nothing.
    /// Binding it to another namespace yields `Error::AmbiguousAbbreviation`.
    pub fn declare(&mut self, abbreviation: &str, namespace: Namespace) -> Result<(), Error> {
        match self.0.get(abbreviation) {
            None => {
                self.0.insert(abbreviation.to_owned(), namespace);
                Ok(())
            }
            Some(bound) if *bound == namespace => Ok(()),
            _ => Err(Error::AmbiguousAbbreviation(abbreviation.to_owned())),
        }
    }

    /// The namespace an abbreviation stands for, or `None` when neither origin binds it.
    pub fn resolve(&self, abbreviation: &str) -> Option<Namespace> {
        self.0.get(abbreviation).copied()
    }
}

impl Default for Abbreviations {
    /// A table of the rule-set abbreviations alone, before a document declares its own.
    fn default() -> Self {
        Self(
            [
                (Namespace::INV.0, Namespace::Invoice),
                (Namespace::CAC.0, Namespace::CommonAggregateComponents),
                (Namespace::CBC.0, Namespace::CommonBasicComponents),
                (Namespace::CII.0, Namespace::CrossIndustryInvoice),
                (
                    Namespace::RAM.0,
                    Namespace::ReusableAggregateBusinessInformationEntity,
                ),
            ]
            .into_iter()
            .map(|(abbreviation, namespace)| (abbreviation.to_owned(), namespace))
            .collect(),
        )
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    #[test]
    fn resolves_an_abbreviation_of_a_rule_set() {
        let abbreviations = Abbreviations::default();

        assert_eq!(abbreviations.resolve("ubl"), Some(Namespace::Invoice));
        assert_eq!(
            abbreviations.resolve("cac"),
            Some(Namespace::CommonAggregateComponents)
        );
        assert_eq!(
            abbreviations.resolve("ram"),
            Some(Namespace::ReusableAggregateBusinessInformationEntity)
        );
    }

    #[test]
    fn resolves_an_abbreviation_of_the_document() {
        let mut abbreviations = Abbreviations::default();

        abbreviations
            .declare("", Namespace::Invoice)
            .expect("a free abbreviation");
        abbreviations
            .declare("basic", Namespace::CommonBasicComponents)
            .expect("a free abbreviation");

        assert_eq!(abbreviations.resolve(""), Some(Namespace::Invoice));
        assert_eq!(
            abbreviations.resolve("basic"),
            Some(Namespace::CommonBasicComponents)
        );
    }

    #[test]
    fn keeps_an_abbreviation_the_document_repeats() {
        let mut abbreviations = Abbreviations::default();

        abbreviations
            .declare("cbc", Namespace::CommonBasicComponents)
            .expect("the namespace the rule sets bind");

        assert_eq!(
            abbreviations.resolve("cbc"),
            Some(Namespace::CommonBasicComponents)
        );
    }

    #[test]
    fn rejects_an_abbreviation_of_two_namespaces() {
        let mut abbreviations = Abbreviations::default();

        let outcome = abbreviations.declare("cbc", Namespace::CommonAggregateComponents);

        assert!(matches!(outcome, Err(Error::AmbiguousAbbreviation(_))));
    }

    #[test]
    fn resolves_no_abbreviation_of_an_unknown_name() {
        assert_eq!(Abbreviations::default().resolve("xsi"), None);
    }
}
