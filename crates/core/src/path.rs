use crate::Error;
use crate::prelude::*;

/// A set of record-form namespaces a binding writes.
///
/// Every member stands for one namespace URI and maps to it and back,
/// so a writer declares the namespaces of the document it emits,
/// and a parser resolves the namespace of an element into a record-form step.
/// The set is known at compile time, so a member is cheap to copy, compare, and hash,
/// which keeps a dictionary key free of owned strings.
/// A member renders as the short abbreviation of its namespace URI.
pub trait Namespace: Copy + Eq + Hash + fmt::Debug + Display {
    /// The full namespace URI the member stands for.
    ///
    /// A `Binding` writes it as the element namespace,
    /// and `Binding::detect` matches a document's root element against the root URIs.
    fn uri(self) -> &'static str;

    /// The member whose URI is `uri`, if the set carries it.
    ///
    /// It is the inverse of `uri`, used by a `Binding` parser
    /// to resolve an element's namespace back into its record-form abbreviation.
    fn from_uri(uri: &str) -> Option<Self>;

    /// The abbreviations the rule sets of the validators write into a location,
    /// bound before the document declares its own.
    fn default_abbreviations() -> Abbreviations<Self>;
}

/// The base set of record-form namespaces: the seven the two bindings write.
///
/// It is the first implementation of `Namespace`, the set `Ubl` and `Cii` carry.
/// The variants cover the two bindings and never mix.
/// The abbreviation stands in for the full namespace in a rendered path,
/// so a UBL path and a CII path never compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum BaseNamespace {
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
    /// The CII Unqualified Data Type namespace,
    /// holding the value carriers such as `udt:DateTimeString`.
    #[display("UDT")]
    UnqualifiedDataType,
    /// The CII Qualified Data Type namespace, holding the formatted value carriers.
    #[display("QDT")]
    QualifiedDataType,
}

impl BaseNamespace {
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
    const UDT: (&str, &str) = (
        "udt",
        "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100",
    );
    const QDT: (&str, &str) = (
        "qdt",
        "urn:un:unece:uncefact:data:standard:QualifiedDataType:100",
    );
}

impl Namespace for BaseNamespace {
    fn uri(self) -> &'static str {
        match self {
            Self::Invoice => Self::INV.1,
            Self::CommonAggregateComponents => Self::CAC.1,
            Self::CommonBasicComponents => Self::CBC.1,
            Self::CrossIndustryInvoice => Self::CII.1,
            Self::ReusableAggregateBusinessInformationEntity => Self::RAM.1,
            Self::UnqualifiedDataType => Self::UDT.1,
            Self::QualifiedDataType => Self::QDT.1,
        }
    }

    fn from_uri(uri: &str) -> Option<Self> {
        match uri {
            _ if uri == Self::INV.1 => Some(Self::Invoice),
            _ if uri == Self::CAC.1 => Some(Self::CommonAggregateComponents),
            _ if uri == Self::CBC.1 => Some(Self::CommonBasicComponents),
            _ if uri == Self::CII.1 => Some(Self::CrossIndustryInvoice),
            _ if uri == Self::RAM.1 => Some(Self::ReusableAggregateBusinessInformationEntity),
            _ if uri == Self::UDT.1 => Some(Self::UnqualifiedDataType),
            _ if uri == Self::QDT.1 => Some(Self::QualifiedDataType),
            _ => None,
        }
    }

    fn default_abbreviations() -> Abbreviations<Self> {
        Abbreviations(
            [
                (Self::INV.0, Self::Invoice),
                (Self::CAC.0, Self::CommonAggregateComponents),
                (Self::CBC.0, Self::CommonBasicComponents),
                (Self::CII.0, Self::CrossIndustryInvoice),
                (
                    Self::RAM.0,
                    Self::ReusableAggregateBusinessInformationEntity,
                ),
                (Self::UDT.0, Self::UnqualifiedDataType),
                (Self::QDT.0, Self::QualifiedDataType),
            ]
            .into_iter()
            .map(|(abbreviation, namespace)| (abbreviation.to_owned(), namespace))
            .collect(),
        )
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
pub struct Abbreviations<N: Namespace>(HashMap<String, N>);

impl<N: Namespace> Abbreviations<N> {
    /// Adds an abbreviation the document binds.
    ///
    /// Rebinding one to the same namespace changes nothing.
    /// Binding it to another namespace yields `Error::AmbiguousAbbreviation`.
    pub fn declare(&mut self, abbreviation: &str, namespace: N) -> Result<(), Error> {
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
    pub fn resolve(&self, abbreviation: &str) -> Option<N> {
        self.0.get(abbreviation).copied()
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

    fn step(namespace: BaseNamespace, name: &str, index: usize) -> Step<BaseNamespace> {
        Step {
            namespace,
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    // The identifier of the second invoice line (`BT-126`) in a UBL document.
    fn ubl_line_id() -> Path<BaseNamespace> {
        Path {
            steps: vec![
                step(BaseNamespace::Invoice, "Invoice", 1),
                step(BaseNamespace::CommonAggregateComponents, "InvoiceLine", 2),
                step(BaseNamespace::CommonBasicComponents, "ID", 1),
            ],
        }
    }

    // The same business term in a CII document, with its own binding vocabulary.
    fn cii_line_id() -> Path<BaseNamespace> {
        Path {
            steps: vec![
                step(
                    BaseNamespace::CrossIndustryInvoice,
                    "CrossIndustryInvoice",
                    1,
                ),
                step(
                    BaseNamespace::CrossIndustryInvoice,
                    "SupplyChainTradeTransaction",
                    1,
                ),
                step(
                    BaseNamespace::ReusableAggregateBusinessInformationEntity,
                    "IncludedSupplyChainTradeLineItem",
                    2,
                ),
                step(
                    BaseNamespace::ReusableAggregateBusinessInformationEntity,
                    "AssociatedDocumentLineDocument",
                    1,
                ),
                step(
                    BaseNamespace::ReusableAggregateBusinessInformationEntity,
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
        assert_eq!(BaseNamespace::Invoice.to_string(), "INV");
        assert_eq!(BaseNamespace::CommonAggregateComponents.to_string(), "CAC");
        assert_eq!(BaseNamespace::CommonBasicComponents.to_string(), "CBC");
        assert_eq!(BaseNamespace::CrossIndustryInvoice.to_string(), "RSM");
        assert_eq!(
            BaseNamespace::ReusableAggregateBusinessInformationEntity.to_string(),
            "RAM"
        );
    }

    #[test]
    fn resolves_an_abbreviation_of_a_rule_set() {
        let abbreviations = BaseNamespace::default_abbreviations();

        assert_eq!(abbreviations.resolve("ubl"), Some(BaseNamespace::Invoice));
        assert_eq!(
            abbreviations.resolve("cac"),
            Some(BaseNamespace::CommonAggregateComponents)
        );
        assert_eq!(
            abbreviations.resolve("ram"),
            Some(BaseNamespace::ReusableAggregateBusinessInformationEntity)
        );
    }

    #[test]
    fn resolves_an_abbreviation_of_the_document() {
        let mut abbreviations = BaseNamespace::default_abbreviations();

        abbreviations
            .declare("", BaseNamespace::Invoice)
            .expect("a free abbreviation");
        abbreviations
            .declare("basic", BaseNamespace::CommonBasicComponents)
            .expect("a free abbreviation");

        assert_eq!(abbreviations.resolve(""), Some(BaseNamespace::Invoice));
        assert_eq!(
            abbreviations.resolve("basic"),
            Some(BaseNamespace::CommonBasicComponents)
        );
    }

    #[test]
    fn keeps_an_abbreviation_the_document_repeats() {
        let mut abbreviations = BaseNamespace::default_abbreviations();

        abbreviations
            .declare("cbc", BaseNamespace::CommonBasicComponents)
            .expect("the namespace the rule sets bind");

        assert_eq!(
            abbreviations.resolve("cbc"),
            Some(BaseNamespace::CommonBasicComponents)
        );
    }

    #[test]
    fn rejects_an_abbreviation_of_two_namespaces() {
        let mut abbreviations = BaseNamespace::default_abbreviations();

        let outcome = abbreviations.declare("cbc", BaseNamespace::CommonAggregateComponents);

        assert!(matches!(outcome, Err(Error::AmbiguousAbbreviation(_))));
    }

    #[test]
    fn resolves_no_abbreviation_of_an_unknown_name() {
        assert_eq!(BaseNamespace::default_abbreviations().resolve("xsi"), None);
    }
}
