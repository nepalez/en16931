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

impl<'a, N: Namespace> FromIterator<(&'a str, N)> for Abbreviations<N> {
    /// Builds the table a namespace set seeds with the abbreviations of the rule sets.
    fn from_iter<I: IntoIterator<Item = (&'a str, N)>>(pairs: I) -> Self {
        Self(
            pairs
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

    #[test]
    fn resolves_an_abbreviation_of_the_document() {
        let mut abbreviations = ubl::Namespace::default_abbreviations();

        abbreviations
            .declare("", ubl::Namespace::Inv)
            .expect("a free abbreviation");
        abbreviations
            .declare("basic", ubl::Namespace::Cbc)
            .expect("a free abbreviation");

        assert_eq!(abbreviations.resolve(""), Some(ubl::Namespace::Inv));
        assert_eq!(abbreviations.resolve("basic"), Some(ubl::Namespace::Cbc));
    }

    #[test]
    fn keeps_an_abbreviation_the_document_repeats() {
        let mut abbreviations = ubl::Namespace::default_abbreviations();

        abbreviations
            .declare("cbc", ubl::Namespace::Cbc)
            .expect("the namespace the rule sets bind");

        assert_eq!(abbreviations.resolve("cbc"), Some(ubl::Namespace::Cbc));
    }

    #[test]
    fn rejects_an_abbreviation_of_two_namespaces() {
        let mut abbreviations = ubl::Namespace::default_abbreviations();

        let outcome = abbreviations.declare("cbc", ubl::Namespace::Cac);

        assert!(matches!(outcome, Err(Error::AmbiguousAbbreviation(_))));
    }

    #[test]
    fn resolves_no_abbreviation_of_an_unknown_name() {
        assert_eq!(ubl::Namespace::default_abbreviations().resolve("xsi"), None);
    }
}
