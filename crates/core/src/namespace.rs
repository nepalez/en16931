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
pub trait Namespace: Copy + Eq + Hash + fmt::Debug + Display + VariantArray {
    /// The full namespace URI the member stands for.
    ///
    /// A `Binding` writes it as the element namespace,
    /// and `Binding::detect` matches a document's root element against the root URIs.
    fn uri(self) -> &'static str;

    /// The XML prefix a document binds to the member when it declares the namespace.
    /// An empty prefix marks the default namespace of the document.
    fn prefix(self) -> &'static str;

    /// The abbreviation the rule sets of the validators write into a location for the member.
    fn abbreviation(self) -> &'static str;

    /// The member whose URI is `uri`, if the set carries it.
    ///
    /// It is the inverse of `uri`, used by a `Binding` parser
    /// to resolve an element's namespace back into its record-form abbreviation.
    fn from_uri(uri: &str) -> Option<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|member| member.uri() == uri)
    }

    /// The abbreviations the rule sets of the validators write into a location,
    /// bound before the document declares its own.
    fn default_abbreviations() -> Abbreviations<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .map(|member| (member.abbreviation(), member))
            .collect()
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

#[cfg(test)]
mod test {
    use super::*;

    use crate::ubl;

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
